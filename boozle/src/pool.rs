use actix::prelude::*;
use tokio::sync::{broadcast, oneshot};

use std::collections::{HashMap};

enum Entry<M>
where
  M: Message + Send,
  M::Result: Send,
{
  Unresolved(broadcast::Sender<Recipient<M>>),
  Resolved(Recipient<M>),
}

#[derive(Debug)]
pub enum InsertUnresolvedError {
  AlreadyExists,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<InsertUnresolvedResult, InsertUnresolvedError>")]
pub struct InsertUnresolved {
  pub id: u64,
}

#[derive(Debug)]
pub struct InsertUnresolvedResult;

#[derive(Debug)]
pub enum ResolveError {
  NotFound,
  AlreadyResolved,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<ResolveResult, ResolveError>")]
pub struct Resolve<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub recipient: Recipient<M>,
}

#[derive(Debug)]
pub struct ResolveResult;

#[derive(Debug)]
pub enum InsertResolvedError {
  AlreadyExists,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<InsertResolvedResult, InsertResolvedError>")]
pub struct InsertResolved<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub recipient: Recipient<M>,
}

#[derive(Debug)]
pub struct InsertResolvedResult;

#[derive(Debug)]
pub enum RouteError {
  NotFound,
  Unresolvable,
  SendFailure,
  Internal
}

#[derive(Message, Debug)]
#[rtype(result = "Result<RouteResult<M>, RouteError>")]
pub struct Route<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub message: M,
}

#[derive(Debug)]
pub struct RouteResult<M>
where
  M: Message + Send,
  M::Result: Send,
{
  pub result: M::Result,
}

#[derive(Debug)]
pub enum RemoveError {
  NotFound,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<RemoveResult<M>, RemoveError>")]
pub struct Remove<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  pub id: u64,
  pub phantom: std::marker::PhantomData<M>,
}

#[derive(Debug)]
pub struct RemoveResult<M>
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
  entries: HashMap<u64, Entry<M>>,
}

impl<M> Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send
{
  pub fn new() -> Self {
    Self {
      entries: HashMap::new()
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

impl<M> Handler<InsertUnresolved> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<InsertUnresolvedResult, InsertUnresolvedError>;

  fn handle(&mut self, msg: InsertUnresolved, _: &mut Context<Self>) -> Self::Result {
    if self.entries.contains_key(&msg.id) {
      return Err(InsertUnresolvedError::AlreadyExists);
    }
    let (tx, _) = broadcast::channel(1);
    self.entries.insert(msg.id, Entry::Unresolved(tx));
    Ok(InsertUnresolvedResult {})
  }
}

impl<M> Handler<Resolve<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<ResolveResult, ResolveError>;

  fn handle(&mut self, msg: Resolve<M>, _: &mut Context<Self>) -> Self::Result {
    let recipient = msg.recipient;
    let prev = self.entries.insert(msg.id, Entry::Resolved(recipient.clone()));

    if let Some(entry) = prev {
      if let Entry::Unresolved(tx) = entry {
        let _ = tx.send(recipient);
        Ok(ResolveResult {})
      } else {
        self.entries.insert(msg.id, entry);
        Err(ResolveError::AlreadyResolved)
      }
    } else {
      self.entries.remove(&msg.id);
      Err(ResolveError::NotFound)
    }
  }
}

impl<M> Handler<InsertResolved<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<InsertResolvedResult, InsertResolvedError>;

  fn handle(&mut self, msg: InsertResolved<M>, _: &mut Context<Self>) -> Self::Result {
    if self.entries.contains_key(&msg.id) {
      return Err(InsertResolvedError::AlreadyExists);
    }

    self.entries.insert(msg.id, Entry::Resolved(msg.recipient));
    Ok(InsertResolvedResult {})
  }
}

impl<M> Handler<Remove<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = Result<RemoveResult<M>, RemoveError>;

  fn handle(&mut self, msg: Remove<M>, _: &mut Context<Self>) -> Self::Result {
    match self.entries.remove(&msg.id) {
      Some(entry) => Ok(RemoveResult {
        recipient: match entry {
          Entry::Unresolved(_) => None,
          Entry::Resolved(recipient) => Some(recipient),
        },
      }),
      None => Err(RemoveError::NotFound),
    }
  }
}

impl<M> Handler<Route<M>> for Pool<M>
where
  M: 'static + Message + Send,
  M::Result: Send,
{
  type Result = ResponseActFuture<Self, Result<RouteResult<M>, RouteError>>;

  fn handle(&mut self, msg: Route<M>, _: &mut Context<Self>) -> Self::Result {
    let (tx, rx) = oneshot::channel();
    let Route { id, message } = msg;

    if let Some(entry) = self.entries.get(&id) {
      match entry {
        Entry::Resolved(recipient) => {
          let arecipient = recipient.clone();
          actix::spawn(async move {
            let _ = match arecipient.send(message).await {
              Ok(res) => tx.send(Ok(RouteResult {result: res })),
              Err(_) => tx.send(Err(RouteError::SendFailure))
            };
          });
        },
        Entry::Unresolved(resolve_tx) => {
          let mut rx = resolve_tx.subscribe();
          actix::spawn(async move {
            let _ = match rx.recv().await {
              Ok(recipient) => {
                match recipient.send(message).await {
                  Ok(res) => tx.send(Ok(RouteResult { result: res })),
                  Err(_) => tx.send(Err(RouteError::SendFailure))
                }
              },
              Err(_) => tx.send(Err(RouteError::Unresolvable))
            };
          });
        }
      }
    } else {
      let _ = tx.send(Err(RouteError::NotFound));
    }

    let fut = async move {
      match rx.await {
        Ok(res) => res,
        Err(_) => Err(RouteError::Internal)
      }
    }.into_actor(self);

    Box::pin(fut)
  }
}
