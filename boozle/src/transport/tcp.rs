// use actix::prelude::*;
// use tokio::net::TcpStream;

// use super::super::multiplexer::{Multiplexer, Request, RequestError, Response, ResponseError};
// use super::Transport;

// use bytes::BytesMut;
// use std::sync::Arc;
// use tokio::io::AsyncWrite;
// use tokio::io::AsyncWriteExt;
// use tokio::sync::Mutex;

// pub struct Tcp {
//   stream: Arc<Mutex<TcpStream>>,
//   multiplexer: Option<Addr<Multiplexer>>,
// }

// pub enum WriteError {
//   SerializationFailed,
//   Io(std::io::Error),
// }

// impl Tcp {
//   pub fn new(stream: TcpStream) -> Self {
//     Self {
//       stream: Mutex::new(stream),
//       multiplexer: None,
//     }
//   }

//   fn message<T: serde::Serialize>(message: &T) -> Result<Vec<u8>, serde_json::Error> {
//     let json = match serde_json::to_vec(message) {
//       Ok(json) => json,
//       Err(err) => return Err(err),
//     };

//     let mut buf = Vec::new();
//     buf.extend_from_slice(&[1u8]);
//     buf.extend_from_slice(&json.len().to_be_bytes());
//     buf.extend_from_slice(json.as_slice());

//     Ok(buf)
//   }
// }

// impl Transport for Tcp {
//   fn set_multiplexer(&mut self, multiplexer: Option<Addr<Multiplexer>>) {
//     self.multiplexer = multiplexer;
//   }
// }

// impl Actor for Tcp {
//   type Context = Context<Self>;
// }

// impl Handler<Request> for Tcp {
//   type Result = ResponseActFuture<Self, Result<(), RequestError>>;

//   fn handle(&mut self, msg: Request, _: &mut Context<Self>) -> Self::Result {
//     let buf = Self::message(&msg).unwrap();
//     let lock = self.stream.lock();
//     let fut = actix::fut::wrap_future::<_, Self>(lock);

//     let fut =
//       fut.then(|stream, actor, _| actix::fut::wrap_future::<_, Self>(stream.write(buf.as_slice())));
//     let fut = fut.map(|value, actor, _| match value {
//       Ok(size) => Ok(()),
//       Err(err) => Err(RequestError::Io(err)),
//     });

//     Box::pin(fut)
//   }
// }

// impl Handler<Response> for Tcp {
//   type Result = ResponseActFuture<Self, Result<(), ResponseError>>;

//   fn handle(&mut self, msg: Response, _: &mut Context<Self>) -> Self::Result {
//     let buf = Self::message(&msg).unwrap();
//     let stream = self.stream;
//     let lock = stream.lock();
//     let fut = actix::fut::wrap_future::<_, Self>(lock);

//     let fut =
//       fut.then(|stream, actor, _| actix::fut::wrap_future::<_, Self>(stream.write(buf.as_slice())));
//     let fut = fut.map(|value, actor, _| match value {
//       Ok(size) => Ok(()),
//       Err(err) => Err(ResponseError::Io(err)),
//     });

//     Box::pin(fut)
//   }
// }
