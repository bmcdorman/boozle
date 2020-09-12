use crossbeam::channel::{bounded, unbounded, Receiver, Sender};

pub type Side<S, R> = (Sender<S>, Receiver<R>);

pub fn unbounded_duplex<L, R>() -> (Side<L, R>, Side<R, L>) {
  let (tx1, rx1) = unbounded();
  let (tx2, rx2) = unbounded();

  ((tx1, rx2), (tx2, rx1))
}

pub fn bounded_duplex<L, R>(cap: usize) -> (Side<L, R>, Side<R, L>) {
  let (tx1, rx1) = bounded(cap);
  let (tx2, rx2) = bounded(cap);

  ((tx1, rx2), (tx2, rx1))
}
