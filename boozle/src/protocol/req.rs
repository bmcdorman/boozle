use crate::value::Value;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Call {
  pub mutable: bool,
  pub object_id: u64,
  pub method_id: u64,
  pub argument: Option<Value>,
  pub to_object_id: Option<u64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Free {
  pub object_id: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Ty {
  Call(Call),
  Free(Free)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Req {
  pub id: u64,
  pub ty: Ty
}