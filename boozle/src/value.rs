use actix::Addr;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::any::Any;
use std::sync::Arc;

use super::object::{Object, ObjectActor};

use actix::Actor;


#[derive(Debug, Serialize, Deserialize)]
pub enum Owner {
  Local,
  Remote,
}

#[derive(Debug)]
pub struct Lit(pub(crate) Arc<[u8]>);

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
  Lit(Lit),
  Ref { owner: u32, id: u64 },
}

impl Serialize for Lit {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer
  {
    self.0.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Lit {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::de::Deserializer<'de>
  {
    Box::<[u8]>::deserialize(deserializer).map(|b| Self(b.into()))
  }
}

pub enum LocalValue {
  Lit(Arc<[u8]>),
  Actor(Addr<ObjectActor>)
}

impl LocalValue {
  pub fn lit(value: Box<[u8]>) -> Self {
    Self::Lit(value.into())
  }

  pub fn from_lit<T: Serialize>(value: &T) -> Self {
    Self::Lit(bincode::serialize(value).unwrap().into_boxed_slice().into())
  }

  pub fn from_object<O: Object + Send + Sync + 'static>(object: O) -> Self {
    Self::Actor(ObjectActor::new(object).start())
  }
}

impl Clone for LocalValue {
  fn clone(&self) -> Self {
    match &self {
      &Self::Lit(x) => Self::Lit(x.clone()),
      &Self::Actor(actor) => Self::Actor(actor.clone()),
    }
  }
}

impl std::fmt::Debug for LocalValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match &self {
        &Self::Lit(x) => return write!(f, "LocalValue::Lit"),
        &Self::Actor(x) => return write!(f, "LocalValue::Actor"),
      }
  }
}