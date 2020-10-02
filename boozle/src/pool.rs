use actix::prelude::*;
use tokio::sync::{broadcast, oneshot};

use std::collections::{HashMap};

/// This enum wraps a value in a Pool's key/value store.
/// Each value can be in either a resolved or unresolved state.
enum Entry<T>
where
  T: Send + Sync
{
  Unresolved(broadcast::Sender<T>),
  Resolved(T),
}

#[derive(Debug)]
pub enum InsertUnresolvedError {
  AlreadyExists,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<InsertUnresolvedResult, InsertUnresolvedError>")]
pub struct InsertUnresolved<K>
where
  K: Key
{
  pub key: K,
}

impl<K: Key> InsertUnresolved<K> {
  pub fn new(key: K) -> Self {
    Self {
      key
    }
  }
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
pub struct Resolve<K, T>
where
  K: Key
{
  key: K,
  value: T,
}

impl<K, T> Resolve<K, T>
where
  K: Key
{
  pub fn new(key: K, value: T) -> Self {
    Self {
      key,
      value
    }
  }

  pub fn key(&self) -> &K {
    &self.key
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  pub fn value_mut(&mut self) -> &mut T {
    &mut self.value
  }

  pub fn take_value(self) -> T {
    self.value
  }
}

#[derive(Debug)]
pub struct ResolveResult;

#[derive(Debug)]
pub enum ExposeError {
  OutOfMemory
}

#[derive(Message, Debug)]
#[rtype(result = "Result<ExposeResult<K>, ExposeError>")]
pub struct Expose<K, T>
where
  K: 'static + Key
{
  pub value: T,
  phantom: std::marker::PhantomData<K>
}

impl<K, T> Expose<K, T>
where
  K: 'static + Key
{
  pub fn new(value: T) -> Self {
    Self {
      value,
      phantom: std::marker::PhantomData {}
    }
  }
}

#[derive(Debug)]
pub struct ExposeResult<K>
where
  K: Key
{
  pub key: K
}

#[derive(Debug)]
pub enum InsertResolvedError {
  AlreadyExists,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<InsertResolvedResult, InsertResolvedError>")]
pub struct InsertResolved<K, T>
where
  K: Key
{
  pub key: K,
  pub value: T,
}

impl<K, T> InsertResolved<K, T>
where
  K: Key
{
  pub fn new(key: K, value: T) -> Self {
    Self {
      key,
      value
    }
  }

  pub fn key(&self) -> &K {
    &self.key
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  pub fn value_mut(&mut self) -> &mut T {
    &mut self.value
  }

  pub fn take_value(self) -> T {
    self.value
  }
}

#[derive(Debug)]
pub struct InsertResolvedResult;

#[derive(Debug)]
pub enum GetError {
  NotFound,
  Unresolvable,
  SendFailure,
  Internal
}

#[derive(Message, Debug)]
#[rtype(result = "Result<T, GetError>")]
pub struct Get<K, T>
where
  K: Key,
  T: 'static
{
  key: K,
  phantom: std::marker::PhantomData<T>
}

impl<K, T> Get<K, T>
where
  K: Key
{
  pub fn new(key: K) -> Self {
    Self {
      key,
      phantom: std::marker::PhantomData
    }
  }

  pub fn key(&self) -> &K {
    &self.key
  }
}

#[derive(Debug)]
pub struct GetResult<T> {
  pub value: T,
}

#[derive(Debug)]
pub enum RemoveError {
  NotFound,
}

#[derive(Message, Debug)]
#[rtype(result = "Result<RemoveResult<T>, RemoveError>")]
pub struct Remove<K, T>
where
  K: Key,
  T: 'static
{
  pub key: K,
  pub phantom: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub struct RemoveResult<T>
{
  /// The removed value, if any
  pub value: Option<T>,
}

/// A Key is a type that can serve as a key in a Pool's key/value store.
pub trait Key: std::fmt::Debug + Clone + Copy + std::hash::Hash + PartialEq + Eq + Sync + Send + std::marker::Unpin {
  const MAX: Self;
  const MIN: Self;
  fn inc(&self) -> Self;
  fn dec(&self) -> Self;
}

/// Pool is an actor that functions as an asynchronous key/value store.
/// 
/// Keys may be in two states: Resolved or Unresolved. If a key is requested with
/// `Get` and it is unresolved, the get will wait for the key to be resolved. If
/// it is resolved, it is returned immediately.
pub struct Pool<K, T>
where
  K: Key,
  T: Send + Sync
{
  expose_iter: K,
  entries: HashMap<K, Entry<T>>,
}

impl<K, T> Pool<K, T>
where
  K: Key,
  T: Send + Sync
{
  pub fn new() -> Self {
    Self {
      expose_iter: K::MAX,
      entries: HashMap::new()
    }
  }
}

impl<K, T: 'static> Actor for Pool<K, T>
where
  K: 'static + Key,
  T: Send + Sync + std::marker::Unpin
{
  type Context = Context<Self>;
}

impl<K, T> Handler<InsertUnresolved<K>> for Pool<K, T>
where
  K: 'static + Key,
  T: 'static + Sync + Send + std::marker::Unpin
{
  type Result = Result<InsertUnresolvedResult, InsertUnresolvedError>;

  fn handle(&mut self, msg: InsertUnresolved<K>, _: &mut Context<Self>) -> Self::Result {
    if self.entries.contains_key(&msg.key) {
      return Err(InsertUnresolvedError::AlreadyExists);
    }
    let (tx, _) = broadcast::channel(1);
    self.entries.insert(msg.key, Entry::Unresolved(tx));
    Ok(InsertUnresolvedResult {})
  }
}

impl<K, T> Handler<Resolve<K, T>> for Pool<K, T>
where
  K: 'static + Key,
  T: 'static + Sync + Send + Clone + std::marker::Unpin
{
  type Result = Result<ResolveResult, ResolveError>;

  fn handle(&mut self, msg: Resolve<K, T>, _: &mut Context<Self>) -> Self::Result {
    let value = msg.value;
    let prev = self.entries.insert(msg.key, Entry::Resolved(value.clone()));

    if let Some(entry) = prev {
      if let Entry::Unresolved(tx) = entry {
        // We were previously unresolved. Send this newly resolved value to all listeners (such as get requests).
        let _ = tx.send(value);
        Ok(ResolveResult {})
      } else {
        // We were already resolved. Re-insert the previous value and return an error.
        self.entries.insert(msg.key, entry);
        Err(ResolveError::AlreadyResolved)
      }
    } else {
      // The user tried to resolve a key that doesn't exist (wasn't marked as unresolved)
      self.entries.remove(&msg.key);
      Err(ResolveError::NotFound)
    }
  }
}

impl<K, T> Handler<Expose<K, T>> for Pool<K, T>
where
  K: 'static + Key,
  T: 'static + Send + Sync + std::marker::Unpin
{
  type Result = Result<ExposeResult<K>, ExposeError>;

  fn handle(&mut self, msg: Expose<K, T>, _: &mut Context<Self>) -> Self::Result {
    let value = msg.value;
    let key = self.expose_iter;
    self.expose_iter.dec();
    self.entries.insert(key, Entry::Resolved(value));
    Ok(ExposeResult { key })
  }
}

impl<K, T> Handler<InsertResolved<K, T>> for Pool<K, T>
where
  K: 'static + Key,
  T: 'static + Send + Sync + std::marker::Unpin
{
  type Result = Result<InsertResolvedResult, InsertResolvedError>;

  fn handle(&mut self, msg: InsertResolved<K, T>, _: &mut Context<Self>) -> Self::Result {
    if self.entries.contains_key(&msg.key) {
      return Err(InsertResolvedError::AlreadyExists);
    }

    self.entries.insert(msg.key, Entry::Resolved(msg.value));
    Ok(InsertResolvedResult {})
  }
}

impl<K, T> Handler<Remove<K, T>> for Pool<K, T>
where
  K: 'static + Key,
  T: 'static + Send + Sync + std::marker::Unpin
{
  type Result = Result<RemoveResult<T>, RemoveError>;

  fn handle(&mut self, msg: Remove<K, T>, _: &mut Context<Self>) -> Self::Result {
    println!("Remove {:?}", msg.key);
    match self.entries.remove(&msg.key) {
      Some(entry) => Ok(RemoveResult {
        value: match entry {
          Entry::Unresolved(_) => None,
          Entry::Resolved(value) => Some(value),
        },
      }),
      None => Err(RemoveError::NotFound),
    }
  }
}

impl<K, T> Handler<Get<K, T>> for Pool<K, T>
where
  K: Key + 'static,
  T: 'static + Send + Sync + Clone + std::marker::Unpin
{
  type Result = ResponseFuture<Result<T, GetError>>;

  fn handle(&mut self, msg: Get<K, T>, _: &mut Context<Self>) -> Self::Result {
    // We have to use some indirection here to express get's asynchronous behavior.
    let (tx, rx) = oneshot::channel();
    let Get { key, phantom: std::marker::PhantomData {} } = msg;

    if let Some(entry) = self.entries.get(&key) {
      match entry {
        Entry::Resolved(value) => {
          // The value is already resolved. Return it.
          let value = value.clone();
          let _ = tx.send(Ok(value));
        },
        Entry::Unresolved(resolve_tx) => {
          // The value is unresolved. Asynchronously listen for it to be resolved.
          let mut rx = resolve_tx.subscribe();
          actix::spawn(async move {
            let _ = match rx.recv().await {
              Ok(value) => tx.send(Ok(value)),
              Err(_) => tx.send(Err(GetError::Unresolvable))
            };
          });
        }
      }
    } else {
      let _ = tx.send(Err(GetError::NotFound));
    }

    let fut = async move {
      match rx.await {
        Ok(res) => res,
        Err(_) => Err(GetError::Internal)
      }
    };

    Box::pin(fut)
  }
}

impl Key for u64 {
  const MIN: Self = std::u64::MIN;
  const MAX: Self = std::u64::MAX;

  fn inc(&self) -> Self {
    self + 1
  }

  fn dec(&self) -> Self {
    self - 1
  }
}

impl Key for u32 {
  const MIN: Self = std::u32::MIN;
  const MAX: Self = std::u32::MAX;

  fn inc(&self) -> Self {
    self + 1
  }

  fn dec(&self) -> Self {
    self - 1
  }
}

#[async_trait::async_trait]
pub trait PoolHelpers<K, T>
where
  K: 'static + Key,
  T: 'static + Send + Sync + std::marker::Unpin
{
  async fn insert_unresolved(&self, key: K) -> Result<InsertUnresolvedResult, InsertUnresolvedError>;
  async fn insert_resolved(&self, key: K, value: T) -> Result<InsertResolvedResult, InsertResolvedError>;

  async fn resolve(&self, key: K, value: T) -> Result<ResolveResult, ResolveError>;
  async fn get(&self, key: K) -> Result<T, GetError>;
  async fn expose(&self, value: T) -> Result<ExposeResult<K>, ExposeError>;
  async fn remove(&self, key: K) -> Result<RemoveResult<T>, RemoveError>;
}

#[async_trait::async_trait]
impl<K, T> PoolHelpers<K, T> for Addr<Pool<K, T>>
where
  K: 'static + Key,
  T: 'static + Send + Sync + std::marker::Unpin + Clone
{
  async fn insert_unresolved(&self, key: K) -> Result<InsertUnresolvedResult, InsertUnresolvedError> {
    self.send(InsertUnresolved::new(key)).await.unwrap()
  }
  
  async fn insert_resolved(&self, key: K, value: T) -> Result<InsertResolvedResult, InsertResolvedError> {
      self.send(InsertResolved::new(key, value)).await.unwrap()
  }

  async fn resolve(&self, key: K, value: T) -> Result<ResolveResult, ResolveError> {
    self.send(Resolve::new(key, value)).await.unwrap()
  }

  async fn get(&self, key: K) -> Result<T, GetError> {
    self.send(Get::new(key)).await.unwrap()
  }

  async fn expose(&self, value: T) -> Result<ExposeResult<K>, ExposeError> {
    self.send(Expose::new(value)).await.unwrap()
  }

  async fn remove(&self, key: K) -> Result<RemoveResult<T>, RemoveError> {
    self.send(Remove {
      key,
      phantom: std::marker::PhantomData {}
    }).await.unwrap()
  }
}