use crate::value::Value;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Return {
  pub value: Option<Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Ty {
  Call(Return),
  Free
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Res {
  pub id: u64,
  pub ty: Ty
}