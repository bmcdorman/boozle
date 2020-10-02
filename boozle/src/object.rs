use actix::prelude::*;
use tokio::sync::{oneshot, RwLock};

use std::sync::Arc;
use std::cell::RefCell;

use super::value::LocalValue;

pub struct Ptr {
  owner: u32,
  id: u64
}

#[derive(Debug)]
pub enum CallError {
  InvalidResponse,
  Failed,
  Comm,
  UnknownMethod,
  InvalidArgument
}

#[derive(Message)]
#[rtype(result = "Result<Return, CallError>")]
pub struct Call {
  pub method_id: u64,
  pub argument: Option<LocalValue>,
}

impl Call {
  pub fn new(method_id: u64, argument: Option<LocalValue>) -> Self {
    Self {
      method_id,
      argument
    }
  }
}

#[derive(Message)]
#[rtype(result = "Result<Return, CallError>")]
pub struct CallMut {
  pub method_id: u64,
  pub argument: Option<LocalValue>,
}

pub struct Return {
  pub result: Option<LocalValue>,
}

pub struct ProxyInfo {

}

#[async_trait::async_trait]
pub trait Object: std::fmt::Debug + Send {
  async fn call(&self, call: Call) -> Result<Return, CallError>;
  async fn call_mut(&mut self, call_mut: CallMut) -> Result<Return, CallError>;

  fn proxy_info(&self) -> Option<ProxyInfo>;
}

#[derive(Message)]
#[rtype(result = "Option<ProxyInfo>")]
pub struct GetProxyInfo {

}

pub struct ObjectActor {
  backing: Arc<RwLock<dyn Object + Send + Sync + 'static>>,
}

impl Actor for ObjectActor {
  type Context = Context<Self>;
}

impl ObjectActor {
  pub fn new<O: Object + Send + Sync + 'static>(backing: O) -> Self {
    Self {
      backing: Arc::new(RwLock::new(backing))
    }
  }
}

impl Handler<Call> for ObjectActor {
  type Result = ResponseFuture<Result<Return, CallError>>;

  fn handle(&mut self, msg: Call, _: &mut Context<Self>) -> Self::Result {
    let backing = self.backing.clone();
    
    Box::pin(async move {
      backing.read().await.call(msg).await
    })
  }
}

impl Handler<CallMut> for ObjectActor {
  type Result = ResponseFuture<Result<Return, CallError>>;

  fn handle(&mut self, msg: CallMut, _: &mut Context<Self>) -> Self::Result {
    let backing = self.backing.clone();
    Box::pin(async move {
      backing.write().await.call_mut(msg).await
    })
  }
}

impl Handler<GetProxyInfo> for ObjectActor {
  type Result = ResponseFuture<Option<ProxyInfo>>;

  fn handle(&mut self, _: GetProxyInfo, _: &mut Context<Self>) -> Self::Result {
    let backing = self.backing.clone();
    Box::pin(async move {
      backing.read().await.proxy_info()
    })
  }
}

#[async_trait::async_trait]
pub trait ObjectActorHelpers {
  async fn call(&self, call: Call) -> Result<Return, CallError>;
  async fn call_mut(&self, call: CallMut) -> Result<Return, CallError>;
  async fn proxy_info(&self) -> Option<ProxyInfo>;
}

#[async_trait::async_trait]
impl ObjectActorHelpers for Addr<ObjectActor> {
  async fn call(&self, call: Call) -> Result<Return, CallError> {
    self.send(call).await.unwrap()
  }

  async fn call_mut(&self, call: CallMut) -> Result<Return, CallError> {
    self.send(call).await.unwrap()
  }

  async fn proxy_info(&self) -> Option<ProxyInfo> {
    self.send(GetProxyInfo {}).await.unwrap()
  }
}

use std::collections::HashSet;

pub struct Access<T> {
  users: HashSet<u64>,
  value: T
}

