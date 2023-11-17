use std::{sync::Arc, time::Duration};

use crossbeam::channel::{unbounded, Receiver, RecvTimeoutError, Sender, TryRecvError};
use parking_lot::{Mutex, RwLock};

pub struct UBChannel<T> {
  pub tx: Arc<Mutex<Sender<T>>>,
  pub rx: Receiver<T>,
}

impl<T> UBChannel<T> {
  pub fn new() -> Self {
    let (tx, rx) = unbounded::<T>();
    Self {
      tx: Arc::new(Mutex::new(tx)),
      rx: rx,
    }
  }

  pub fn share_tx(&self) -> Arc<Mutex<Sender<T>>> {
    self.tx.clone()
  }

  pub fn send(&self, msg: T) {
    if let Err(e) = self.tx.lock().send(msg) {
      eprintln!("send fail: {}", e);
    }
  }

  pub fn recv(&self) {
    if let Err(e) = self.rx.recv() {
      eprintln!("recv fail: {}", e);
    }
  }

  pub fn try_recv(&self) -> Result<T, TryRecvError> {
    self.rx.try_recv()
  }

  pub fn recv_timeout(&self, duration: Duration) -> Result<T, RecvTimeoutError> {
    self.rx.recv_timeout(duration)
  }
}

