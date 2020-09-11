use actix::prelude::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Error {}

#[derive(Message)]
#[rtype(result = "Result<InjectResult, Error>")]
pub struct Inject<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub recipient: Recipient<M>,
}

pub struct InjectResult {
  pub id: u64,
}

#[derive(Message)]
#[rtype(result = "Result<RouteResult<M>, Error>")]
pub struct Route<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub message: M,
}

pub struct RouteResult<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub result: M::Result,
}

#[derive(Message)]
#[rtype(result = "Result<EjectResult<M>, Error>")]
pub struct Eject<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub phantom: std::marker::PhantomData<M>,
}

pub struct EjectResult<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub recipient: Option<Recipient<M>>,
}

pub struct Pool<M>
where
  M: Message + Send,
  M::Result: Send,
{
  iter: u64,
  recipients: HashMap<u64, Recipient<M>>,
}

impl<M> Pool<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub fn new() -> Self {
    Self {
      iter: 0,
      recipients: HashMap::new(),
    }
  }
}

impl<M> Actor for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Context = Context<Self>;
}

impl<M> Handler<Inject<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<InjectResult, Error>;

  fn handle(&mut self, msg: Inject<M>, _: &mut Context<Self>) -> Self::Result {
    self.iter += 1;

    self.recipients.insert(self.iter, msg.recipient);

    Ok(InjectResult { id: self.iter })
  }
}

impl<M> Handler<Eject<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<EjectResult<M>, Error>;

  fn handle(&mut self, msg: Eject<M>, _: &mut Context<Self>) -> Self::Result {
    Ok(EjectResult {
      recipient: self.recipients.remove(&msg.id),
    })
  }
}

impl<M> Handler<Route<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = ResponseActFuture<Self, Result<RouteResult<M>, Error>>;

  fn handle(&mut self, msg: Route<M>, _: &mut Context<Self>) -> Self::Result {
    let recipient = self.recipients.get(&msg.id).unwrap();
    let future = recipient.send(msg.message);
    let future = actix::fut::wrap_future::<_, Self>(future);
    let wrap = future.map(|result, _actor, _ctx| match result {
      Ok(result) => Ok(RouteResult { result }),
      Err(_err) => Err(Error {}),
    });
    Box::pin(wrap)
  }
}
