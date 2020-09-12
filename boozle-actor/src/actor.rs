
use crate::addr::ActorEvent;
use crate::runtime::spawn;
use crate::{Addr, Context};
use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::oneshot;
use futures::{FutureExt, StreamExt};

/// Represents a message that can be handled by the actor.
pub trait Message: 'static + Send {
    /// The return value type of the message
    /// This type can be set to () if the message does not return a value, or if it is a notification message
    type Result: 'static + Send;
}

/// Describes how to handle messages of a specific type.
/// Implementing Handler is a general way to handle incoming messages.
/// The type T is a message which can be handled by the actor.
#[async_trait::async_trait]
pub trait Handler<T: Message>: Actor {
    /// Method is called for every message received by this Actor.
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: T) -> T::Result;
}

/// Actors are objects which encapsulate state and behavior.
/// Actors run within a specific execution context `Context<A>`.
/// The context object is available only during execution.
/// Each actor has a separate execution context.
///
/// Roles communicate by exchanging messages.
/// The requester can wait for a response.
/// By `Addr` referring to the actors, the actors must provide an `Handle<T>` implementation for this message.
/// All messages are statically typed.
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait Actor: Sized + Send + 'static {
    /// Called when the actor is first started.
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }

    /// Called after an actor is stopped.
    async fn stopped(&mut self, ctx: &mut Context<Self>) {}

    /// Start a new actor, returning its address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xactor::*;
    ///
    /// struct MyActor;
    ///
    /// impl Actor for MyActor {}
    ///
    /// #[message(result = "i32")]
    /// struct MyMsg(i32);
    ///
    /// #[async_trait::async_trait]
    /// impl Handler<MyMsg> for MyActor {
    ///     async fn handle(&mut self, _ctx: &mut Context<Self>, msg: MyMsg) -> i32 {
    ///         msg.0 * msg.0
    ///     }
    /// }
    ///
    /// #[xactor::main]
    /// async fn main() -> Result<()> {
    ///     // Start actor and get its address
    ///     let mut addr = MyActor.start().await?;
    ///
    ///     // Send message `MyMsg` to actor via addr
    ///     let res = addr.call(MyMsg(10)).await?;
    ///     assert_eq!(res, 100);
    ///     Ok(())
    /// }
    /// ```
    async fn start(self) -> Result<Addr<Self>> {
        let (tx_exit, rx_exit) = oneshot::channel();
        let rx_exit = rx_exit.shared();
        let (ctx, rx, tx) = Context::new(Some(rx_exit));
        let rx_exit = ctx.rx_exit.clone();
        let actor_id = ctx.actor_id();
        start_actor(ctx, rx, tx_exit, self).await?;
        Ok(Addr {
            actor_id,
            tx,
            rx_exit,
        })
    }
}

pub(crate) async fn start_actor<A: Actor>(
    mut ctx: Context<A>,
    mut rx: UnboundedReceiver<ActorEvent<A>>,
    tx_exit: oneshot::Sender<()>,
    mut actor: A,
) -> Result<()> {
    // Call started
    actor.started(&mut ctx).await?;

    spawn({
        async move {
            while let Some(event) = rx.next().await {
                match event {
                    ActorEvent::Exec(f) => f(&mut actor, &mut ctx).await,
                    ActorEvent::Stop(_err) => break,
                    ActorEvent::RemoveStream(id) => {
                        if ctx.streams.contains(id) {
                            ctx.streams.remove(id);
                        }
                    }
                }
            }

            actor.stopped(&mut ctx).await;

            for (_, handle) in ctx.streams.iter() {
                handle.abort();
            }

            tx_exit.send(()).ok();
        }
    });

    Ok(())
}
