pub type Side<S, R> = (
  crossbeam::channel::Sender<S>,
  crossbeam::channel::Receiver<R>,
);

pub fn unbounded_duplex<L, R>() -> (Side<L, R>, Side<R, L>) {
  let (tx1, rx1) = crossbeam::channel::unbounded();
  let (tx2, rx2) = crossbeam::channel::unbounded();

  ((tx1, rx2), (tx2, rx1))
}

pub fn bounded_duplex<L, R>(cap: usize) -> (Side<L, R>, Side<R, L>) {
  let (tx1, rx1) = crossbeam::channel::bounded(cap);
  let (tx2, rx2) = crossbeam::channel::bounded(cap);

  ((tx1, rx2), (tx2, rx1))
}
