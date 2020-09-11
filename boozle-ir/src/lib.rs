// use std::collections::HashMap;

// mod builtin;
// mod value;

// // use value::Value;

// pub struct RemoteId(u32);
// pub struct MethodId(u32);
// pub struct PromiseId(u32);

// impl From<u32> for RemoteId {
//   fn from(value: u32) -> Self {
//     Self(value)
//   }
// }

// impl From<u32> for MethodId {
//   fn from(value: u32) -> Self {
//     Self(value)
//   }
// }

// impl From<u32> for PromiseId {
//   fn from(value: u32) -> Self {
//     Self(value)
//   }
// }

// pub enum Expr {
//   Method(Box<Expr>, MethodId),
//   Call {
//     method: Box<Expr>,
//     argument: Box<Expr>,
//   },
//   Value(Value),
//   Promise(PromiseId),
// }

// impl From<Value> for Expr {
//   fn from(value: Value) -> Self {
//     Self::Value(value.into())
//   }
// }

// impl From<Value> for Box<Expr> {
//   fn from(value: Value) -> Self {
//     Box::new(Expr::Value(value.into()))
//   }
// }

// fn test() {
//   let call = Expr::Call {
//     method: Box::new(Expr::Method(
//       Value::Object(Object::new(0, 1)).into(),
//       1u32.into(),
//     )),
//     argument: Value::U8(1u8).into(),
//   };
// }

// pub fn segment(expr: Expr) -> HashMap<RemoteId, (PromiseId, Expr)> {
//   HashMap::new()
// }

// pub struct Object {
//   owner: u32,
//   addr: u32,
// }

// impl Object {
//   pub fn new(owner: u32, addr: u32) -> Self {
//     Self { owner, addr }
//   }
// }

// pub struct Unit {
//   consts: Vec<Value>,
//   instructions: Vec<Instruction>,
// }

// pub enum Instruction {
//   Push(Value),
//   Pop,
//   Load(u16),
//   Invoke { method_id: u32 },
// }
