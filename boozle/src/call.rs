use actix::Message;

use super::value::Value;

#[derive(Debug)]
pub enum Error {
  InvalidResponse,
  Failed,
  Comm,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Return, Error>")]
pub struct Call {
  pub method_id: u64,
  pub argument: Value,
}

#[derive(Debug)]
pub struct Return {
  pub result: Value,
}

pub type Pool = super::pool::Pool<Call>;
