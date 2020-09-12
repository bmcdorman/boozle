use actix::prelude::*;
use tokio::sync::oneshot;


use serde::{Serialize, Deserialize};

use super::call;
use super::pool::{Route};
use super::value::Value;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Error {}

#[derive(Serialize, Deserialize)]
pub enum RequestData {
  Call {
    object_id: u64,
    method_id: u64,
    argument: Value,
  },
}

#[derive(Debug)]
pub enum RequestError {
  SerializationFailed,
  Io(std::io::Error)
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "Result<(), RequestError>")]
pub struct Request {
  pub id: u64,
  pub data: RequestData,
}

#[derive(Serialize, Deserialize)]
pub enum ResponseData {
  Return { result: Value },
}

pub enum ResponseError {
  InvalidId,
  SerializationFailed,
  Io(std::io::Error)
}

#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "Result<(), ResponseError>")]
pub struct Response {
  pub id: u64,
  pub data: ResponseData,
}

pub enum BindError {}

#[derive(Message)]
#[rtype(result = "Result<(), BindError>")]
pub struct Bind {
  pub addr: Addr<Multiplexer>,
}

#[derive(Message)]
#[rtype(result = "Result<call::Return, call::Error>")]
pub struct RoutedCall {
  pub object_id: u64,
  pub call: call::Call,
}

pub struct Multiplexer {
  request_id_iter: u64,
  request: Recipient<Request>,
  response: Recipient<Response>,
  pool: Addr<call::Pool>,
  pending: HashMap<u64, oneshot::Sender<Result<Response, (u64, Error)>>>,
}

impl Multiplexer {
  pub fn new(pool: Addr<call::Pool>, request: Recipient<Request>, response: Recipient<Response>) -> Self {
    Self {
      request_id_iter: 0,
      request,
      response,
      pool,
      pending: HashMap::new(),
    }
  }
}

impl Actor for Multiplexer {
  type Context = Context<Self>;
}

impl Handler<RoutedCall> for Multiplexer {
  type Result = ResponseActFuture<Self, Result<call::Return, call::Error>>;

  fn handle(&mut self, msg: RoutedCall, _: &mut Context<Self>) -> Self::Result {
    self.request_id_iter += 1;

    let (tx, rx) = oneshot::channel::<Result<Response, (u64, Error)>>();

    let id = self.request_id_iter;

    let rx_fut = actix::fut::wrap_future::<_, Self>(rx).map(|value, actor, _| match value {
      Ok(v) => match v {
        Ok(v) => {
          actor.pending.remove(&v.id);
          match v.data {
            ResponseData::Return { result } => Ok(call::Return { result }),
            _ => Err(call::Error::InvalidResponse),
          }
        }
        Err(err) => {
          actor.pending.remove(&err.0);
          Err(call::Error::Failed)
        }
      },
      Err(_) => Err(call::Error::Comm),
    });

    let fut = actix::fut::wrap_future::<_, Self>(self.request.send(Request {
      id,
      data: RequestData::Call {
        object_id: msg.object_id,
        method_id: msg.call.method_id,
        argument: msg.call.argument,
      },
    }))
    .then(move |value, actor, _| match value {
      Ok(_) => {
        actor.pending.insert(id, tx);
        rx_fut
      }
      Err(_) => {
        let _ = tx.send(Err((id, Error {})));
        rx_fut
      }
    });
    Box::pin(fut)
  }
}

impl Handler<Response> for Multiplexer {
  type Result = Result<(), ResponseError>;

  fn handle(&mut self, msg: Response, _: &mut Context<Self>) -> Self::Result {
    match self.pending.remove(&msg.id) {
      Some(tx) => {
        let _ = tx.send(Ok(msg));
        Ok(())
      }
      None => Err(ResponseError::InvalidId),
    }
  }
}

impl Handler<Request> for Multiplexer {
  type Result = Result<(), RequestError>;

  fn handle(&mut self, msg: Request, _: &mut Context<Self>) -> Self::Result {
    match msg.data {
      RequestData::Call {
        object_id,
        method_id,
        argument,
      } => {
        let pool = self.pool.clone();
        let response = self.response.clone();
        let id = msg.id;
        actix::spawn(async move {
          let send_result = pool.send(Route {
            id: object_id,
            message: call::Call {
              method_id,
              argument,
            },
          }).await;
          match send_result {
            Ok(route_result) => {
              match route_result {
                Ok(route_result) => {
                  match route_result.result {
                    Ok(ret) => {
                      let _ = response.send(Response {
                        id,
                        data: ResponseData::Return { result: ret.result }
                      }).await;
                    },
                    Err(err) => {
                      unimplemented!();
                    }
                  }
                }
                Err(err) => {
                  unimplemented!();
                }
              }
            }
            Err(err) => {
              unimplemented!();
            }
          }
        });
        Ok(())
      }
    }
  }
}