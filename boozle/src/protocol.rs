use serde::{Serialize, Deserialize};

pub mod req;
pub mod res;

pub use req::Req;
pub use res::Res;

#[derive(Debug, Serialize, Deserialize)]
pub enum Dir {
  Req,
  Res
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Msg {
  pub dir: Dir,
  pub data: Box<[u8]>
}

impl Msg {
  pub fn req(data: Box<[u8]>) -> Self {
    Self {
      dir: Dir::Req,
      data
    }
  }

  pub fn res(data: Box<[u8]>) -> Self {
    Self {
      dir: Dir::Res,
      data
    }
  }
}

