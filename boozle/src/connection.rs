use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

use super::object::{ObjectActor, Call, ObjectActorHelpers, CallMut};
use super::pool::{Pool, InsertUnresolved, InsertResolved, Resolve, Get, PoolHelpers};
use super::value::{Value, LocalValue, Lit};
use super::protocol as proto;

use std::sync::{Arc, atomic::AtomicU64};

pub mod req;
pub mod res;
mod remote;


#[derive(Debug)]
struct CompleteResult;

#[derive(Debug)]
enum CompleteError {
  NotFound
}

#[derive(Message)]
#[rtype(result = "Result<(), CompleteError>")]
struct Complete {
  req_id: u64,
  result: Result<res::Res, ()>
}

#[derive(Message)]
#[rtype(result = "Addr<Pool<u64, LocalValue>>")]
struct GetPool;

pub struct Connection {
  request_iter: u64,
  object_iter: Arc<AtomicU64>,
  tx: mpsc::Sender<Box<[u8]>>,
  rx: Option<mpsc::Receiver<Box<[u8]>>>,
  pool: Addr<Pool<u64, LocalValue>>,
  outstanding: HashMap<u64, oneshot::Sender<Result<res::Res, ()>>>,
}

impl Connection {
  pub fn new(tx: mpsc::Sender<Box<[u8]>>, rx: mpsc::Receiver<Box<[u8]>>) -> Self {
    Self {
      request_iter: 0,
      object_iter: Arc::new(AtomicU64::new(0)),
      tx,
      rx: Some(rx),
      pool: Pool::new().start(),
      outstanding: HashMap::new(),
    }
  }

  async fn on_req(mut tx: mpsc::Sender<Box<[u8]>>, addr: Addr<Self>, pool: Addr<Pool<u64, LocalValue>>, req: proto::Req) {
    
    let res = match req.ty {
      proto::req::Ty::Call(call) => {
        let value = pool.get(call.object_id).await.unwrap();
        match value {
          LocalValue::Lit(_) => {
            // panic!("Remote tried to call a method on a literal");
            proto::Res {
              id: req.id,
              ty: proto::res::Ty::Call(proto::res::Return {
                value: None
              })
            }
          },
          LocalValue::Actor(actor) => {
            let argument = match call.argument {
              Some(argument) => Some(match argument {
                Value::Lit(lit) => LocalValue::Lit(lit.0),
                Value::Ref { owner, id } => LocalValue::from_object(remote::Remote::new(id, addr))
              }),
              None => None
            };
            let result = if call.mutable {
              actor.call_mut(CallMut {
                method_id: call.method_id,
                argument,
              }).await.unwrap()
            } else {
              actor.call(Call {
                method_id: call.method_id,
                argument,
              }).await.unwrap()
            };

            let return_value = if let Some(to_object_id) = call.to_object_id {
              match result.result {
                Some(value) => {
                  let key = pool.insert_resolved(to_object_id, value).await.unwrap();
                  Some(Value::Ref { owner: 0, id: to_object_id })
                },
                None => None
              }
            } else {
              match result.result {
                Some(value) => {
                  match value {
                    LocalValue::Actor(_) => None,
                    LocalValue::Lit(lit) => Some(Value::Lit(Lit(lit)))
                  }
                },
                None => None
              }
            };
            
            proto::Res {
              id: req.id,
              ty: proto::res::Ty::Call(proto::res::Return {
                value: return_value
              })
            }
          }
        }
      },
      proto::req::Ty::Free(free) => {
        pool.remove(free.object_id).await.unwrap();
        proto::Res {
          id: req.id,
          ty: proto::res::Ty::Free
        }
      }
    };

    let data = bincode::serialize(&res).unwrap().into_boxed_slice();
    let msg = bincode::serialize(&proto::Msg::res(data)).unwrap().into_boxed_slice();
    tx.send(msg).await.unwrap();
    
  }

  async fn on_res(tx: mpsc::Sender<Box<[u8]>>, addr: Addr<Self>, pool: Addr<Pool<u64, LocalValue>>, res: proto::Res) {
    match res.ty {
      proto::res::Ty::Call(call) => {
        let value = match call.value {
          Some(value) => Some(match value {
            Value::Lit(lit) => LocalValue::Lit(lit.0),
            Value::Ref { owner, id } => LocalValue::from_object(remote::Remote::new(id, addr.clone()))
          }),
          None => None
        };
        addr.send(Complete {
          req_id: res.id,
          result: Ok(res::Res::Call(res::Return {
            value
          }))
        }).await.unwrap().unwrap();
      },
      _ => {}
    }
  }
}

impl Actor for Connection {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    let mut rx = self.rx.take().unwrap();
    let tx = self.tx.clone();

    let pool = self.pool.clone();
    let addr = ctx.address().clone();
    
    actix::spawn(async move {
      loop {
        match rx.recv().await {
          Some(data) => {
            let msg: proto::Msg = bincode::deserialize(&data).unwrap();
            match msg.dir {
              proto::Dir::Req => {
                let req: proto::Req = bincode::deserialize(&msg.data).unwrap();
                actix::spawn(Connection::on_req(tx.clone(), addr.clone(), pool.clone(), req));
              },
              proto::Dir::Res => {
                let res: proto::Res = bincode::deserialize(&msg.data).unwrap();
                actix::spawn(Connection::on_res(tx.clone(), addr.clone(), pool.clone(), res));
              }
            }
          },
          None => {
            break
          }
        }
      }
    })
    
  }
}

impl Handler<req::Req> for Connection {
  type Result = ResponseFuture<Result<res::Res, ()>>;

  fn handle(&mut self, msg: req::Req, _: &mut Context<Self>) -> Self::Result {
    let (tx, rx) = oneshot::channel();

    let pool = self.pool.clone();
    let object_iter = self.object_iter.clone();
    let mut conn_tx = self.tx.clone();
    self.request_iter += 1;
    let request_iter = self.request_iter;
    self.outstanding.insert(request_iter, tx);

    Box::pin(async move {
      let data = match msg {
        req::Req::Call(call) => {
          let argument = match call.argument {
            Some(argument) => {
              Some(match argument {
                LocalValue::Lit(lit) => {
                  Value::Lit(Lit(lit))
                },
                LocalValue::Actor(actor) => {
                  let proxy_info = actor.proxy_info().await;
                  let key = pool.expose(LocalValue::Actor(actor)).await.unwrap().key;

                  Value::Ref {
                    owner: 0,
                    id: key
                  }
                }
              })
            },
            None => None
          };
          let call = proto::req::Call {
            mutable: call.mutable,
            to_object_id: if call.store_result { Some(object_iter.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) } else { None },
            object_id: call.object_id,
            method_id: call.method_id,
            argument
          };

          bincode::serialize(&proto::Req {
            id: request_iter,
            ty: proto::req::Ty::Call(call)
          }).unwrap().into_boxed_slice()
        },
        req::Req::Free(free) => {
          bincode::serialize(&proto::Req {
            id: request_iter,
            ty: proto::req::Ty::Free(proto::req::Free {
              object_id: free.object_id
            })
          }).unwrap().into_boxed_slice()
        }
      };


      
      let msg = bincode::serialize(&proto::Msg::req(data)).unwrap().into_boxed_slice();


      conn_tx.send(msg).await.unwrap();

      // TODO: If send fails, clean up pool allocations

      rx.await.unwrap()
    })
  }
}

impl Handler<Complete> for Connection {
  type Result = Result<(), CompleteError>;

  fn handle(&mut self, complete: Complete, _: &mut Context<Self>) -> Self::Result {
    match self.outstanding.remove(&complete.req_id) {
      Some(tx) => {
        let _ = tx.send(complete.result);
        Ok(())
      },
      None => {
        Err(CompleteError::NotFound)
      }
    }
  }
}

impl Handler<GetPool> for Connection {
  type Result = Addr<Pool<u64, LocalValue>>;

  fn handle(&mut self, msg: GetPool, _: &mut Context<Self>) -> Self::Result {
    self.pool.clone()
  }
}

#[async_trait::async_trait]
pub trait ConnectionHelpers {
  async fn req(&self, req: req::Req) -> Result<res::Res, ()>;
  async fn pool(&self) -> Addr<Pool<u64, LocalValue>>;
}

#[async_trait::async_trait]
impl ConnectionHelpers for Addr<Connection> {
  async fn req(&self, req: req::Req) -> Result<res::Res, ()> {
    self.send(req).await.unwrap()
  }

  async fn pool(&self) -> Addr<Pool<u64, LocalValue>> {
    self.send(GetPool {}).await.unwrap()
  }
}