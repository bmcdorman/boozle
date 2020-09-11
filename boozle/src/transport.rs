use actix::prelude::*;

use super::multiplexer::{Multiplexer, Request, Response};

pub trait Transport: Actor + Handler<Request> + Handler<Response> {
  fn set_multiplexer(&mut self, multiplexer: Option<Addr<Multiplexer>>);
}

pub mod tcp;
