
use actix::Addr;

use super::{Response, Connection, ConnectionHelpers};

use crate::object::{Object, Call, ProxyInfo, CallMut, CallError, Return};
use crate::connection::{req};

pub struct Remote {
  id: u64,
  connection: Addr<Connection>,
}

impl Remote {
  pub fn new(id: u64, connection: Addr<Connection>) -> Self {
    Self { id, connection }
  }
}

#[async_trait::async_trait]
impl Object for Remote {
  async fn call(&self, call: Call) -> Result<Return, CallError> {
    let response = self.connection.send(req::Req::Call(req::Call {
      mutable: false,
      argument: call.argument,
      method_id: call.method_id,
      object_id: self.id,
      store_result: true
    })).await.unwrap();

    Err(CallError::Failed)
  }



  async fn call_mut(&mut self, call: CallMut) -> Result<Return, CallError> {
    let response = self.connection.send(req::Req::Call(req::Call {
      mutable: true,
      argument: call.argument,
      method_id: call.method_id,
      object_id: self.id,
      store_result: true
    })).await.unwrap();

    Err(CallError::Failed)
  }

  fn proxy_info(&self) -> Option<ProxyInfo> {
      None
  }
}

impl std::fmt::Debug for Remote {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Remote")
     .field("id", &self.id)
     .finish()
  }
}

impl Drop for Remote {
  fn drop(&mut self) {
    // FIXME: This naive implementation slows RPC throughput by more than 4000%
    // These should be batched and sent periodically
    let id = self.id;
    let connection = self.connection.clone();
    actix::spawn(async move {
      connection.req(req::Req::Free(req::Free {
        object_id: id
      })).await.unwrap();
    });
  }
}