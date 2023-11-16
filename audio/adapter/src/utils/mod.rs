use std::{sync::Arc};

use parking_lot::Mutex;
use tokio::sync::mpsc::{
  UnboundedSender,
  UnboundedReceiver,
  unbounded_channel,
};

pub struct TokioUnbounded<T> {
  pub tx: Arc<Mutex<UnboundedSender<T>>>,
  pub rx: Arc<Mutex<UnboundedReceiver<T>>>,
}

impl<T> TokioUnbounded<T> {
  pub fn new() -> Self {
    let (tx, rx) = unbounded_channel::<T>();
    TokioUnbounded { tx: Arc::new(Mutex::new(tx)), rx: Arc::new(Mutex::new(rx)) }
  }

  pub fn send(&self, payload: T) {
    let tx = self.tx.clone();
    if let Err(e) = tx.lock().send(payload) {
      eprintln!("send data fail: {}", e);
    };
  }

  pub async fn wait(&self) -> Option<T> {
    self.rx.clone().lock().recv().await
  }
}