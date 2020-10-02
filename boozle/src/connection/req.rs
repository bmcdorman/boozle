use crate::value::LocalValue;
use super::res;
use actix::Message;

#[derive(Message)]
#[rtype(result = "Result<res::Return, ()>")]
pub struct Call {
  pub mutable: bool,
  pub object_id: u64,
  pub method_id: u64,
  pub argument: Option<LocalValue>,
  pub store_result: bool
}

#[derive(Debug, Message)]
#[rtype(result = "Result<res::Free, ()>")]
pub struct Free {
  pub object_id: u64
}

#[derive(Message)]
#[rtype(result = "Result<res::Res, ()>")]
pub enum Req {
  Call(Call),
  Free(Free)
}
