use std::time::Duration;

use erased_serde::Serialize;
use futures_util::StreamExt;
use log::info;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WSOption {
  url: String,
  retry_interval: Duration,
  retry_max_count: Option<u32>,
}

pub struct WSClient {
  transformer: Option<Box<dyn WSTransform>>,
}

impl WSClient {
  pub fn new() -> Self {
    Self { transformer: None }
  }
  pub fn transform(&mut self, t: Box<dyn WSTransform>) {
    self.transformer = Some(t);
  }
  pub async fn connect(&self, option: &WSOption) -> anyhow::Result<()> {
    let mut retry_cnt = 0;

    loop {
      if let Ok((ws_stream, _)) = connect_async(option.url.clone()).await {
        let (sink, stream) = ws_stream.split();
        let () = broadcast::channel::<Message>(128);
      }

      retry_cnt += 1;
      if let Some(retry_max_count) = option.retry_max_count {
        if retry_cnt >= retry_max_count {
          info!("exceed max retry count: {retry_max_count}");
          break;
        }
      }
    }

    Ok(())
  }
}

pub trait WSTransform {
  fn transform(&self, data: Box<dyn Serialize>) -> Box<dyn Serialize>;
}
