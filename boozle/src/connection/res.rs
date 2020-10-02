use crate::value::LocalValue;

pub struct Return {
  pub value: Option<LocalValue>
}

pub struct Free;

pub enum Res {
  Call(Return),
  Free(Free)
}