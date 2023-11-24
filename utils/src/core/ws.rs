use std::{
  fmt::Display,
  time::{Duration, Instant},
};

use anyhow::{anyhow, Result};

use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{
  broadcast,
  mpsc::{self, Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use log::{error, info, warn};

pub struct WSOption {
  pub url: String,
  pub close_after: Duration,
  pub retry_interval: Duration,
  pub retry_max_count: Option<u32>,
}

#[derive(Clone)]
pub struct WSClient<T> {
  transformer: Option<T>,
  emitter: Option<Sender<Message>>,
  handler: broadcast::Sender<EventMessage>,
  last_ping: Instant,
}

enum LoopFlow {
  Break,
  Continue,
}

#[derive(Clone, Debug)]
pub enum EventMessage {
  Open(),
  Close(),
  Text(String),
  Binary(Vec<u8>),
}

impl<T> WSClient<T>
where
  T: WSTransform,
{
  pub fn transform(&mut self, t: T) {
    self.transformer = Some(t);
  }

  pub async fn send<D: Serialize + Display>(&self, data: D) -> Result<()> {
    let emitter = self.emitter.clone().ok_or(anyhow!("no emitter know"))?;
    if let Some(ts) = &self.transformer {
      if let Ok(text) = ts.to_str(data) {
        emitter.send(Message::Text(text)).await?;
      }
    } else {
      emitter
        .send(Message::Text(serde_json::to_string(&data)?))
        .await?;
    }
    Ok(())
  }

  pub async fn inner_recv<O>(&self, rcv: &mut broadcast::Receiver<EventMessage>) -> Result<O>
  where
    O: DeserializeOwned + Display,
  {
    Ok(match rcv.recv().await? {
      EventMessage::Text(text) => {
        if let Some(ts) = &self.transformer {
          ts.from_str::<O>(&text)?
        } else {
          serde_json::from_str::<O>(&text)?
        }
      }
      EventMessage::Binary(bin) => {
        if let Some(ts) = &self.transformer {
          ts.from_slice(&bin)?
        } else {
          serde_json::from_slice::<O>(&bin)?
        }
      }
      EventMessage::Open() => {
        if let Some(ts) = &self.transformer {
          ts.on_open()?
        } else {
          serde_json::from_value::<O>(serde_json::Value::Null)?
        }
      }
      EventMessage::Close() => {
        if let Some(ts) = &self.transformer {
          ts.on_close()?
        } else {
          serde_json::from_value::<O>(serde_json::Value::Null)?
        }
      }
    })
  }

  pub async fn recv<O>(&self) -> Result<T>
  where
    T: DeserializeOwned + Display,
  {
    self.inner_recv(&mut self.handler.subscribe()).await
  }
}

impl<T> WSClient<T> {
  pub fn new() -> Self {
    let (handler, _) = broadcast::channel::<EventMessage>(128);
    Self {
      transformer: None,
      emitter: None,
      handler,
      last_ping: Instant::now(),
    }
  }

  pub fn subscribe(&self) -> broadcast::Receiver<EventMessage> {
    self.handler.subscribe()
  }

  fn handle_stream_message(&mut self, message: Result<Message>) -> Result<LoopFlow> {
    if let Err(e) = message {
      eprintln!("get stream message: {e}");
      return Ok(LoopFlow::Continue);
    }
    match message.unwrap() {
      Message::Text(text) => {
        self.handler.send(EventMessage::Text(text))?;
        Ok(LoopFlow::Continue)
      }
      Message::Binary(bin) => {
        self.handler.send(EventMessage::Binary(bin))?;
        Ok(LoopFlow::Continue)
      }
      Message::Close(_) => {
        info!("receive close, will break");
        Ok(LoopFlow::Break)
      }
      Message::Ping(_) => {
        self.last_ping = Instant::now();
        Ok(LoopFlow::Continue)
      }
      msg => {
        info!("receive message: {msg:?}");
        Ok(LoopFlow::Continue)
      }
    }
  }

  pub async fn connect(&mut self, option: &WSOption) -> anyhow::Result<()> {
    let mut retry_cnt = 0;

    loop {
      self.emitter = None;
      let mut ping_check_interval = tokio::time::interval(Duration::from_secs(1));

      if let Ok((ws_stream, _)) = connect_async(option.url.clone()).await {
        let (sink, mut stream) = ws_stream.split();
        let (emitter, forwarder) = mpsc::channel::<Message>(128);
        self.emitter = Some(emitter);
        let forwarder_stream: ReceiverStream<Message> = forwarder.into();
        tokio::spawn(forwarder_stream.map(|m| Ok(m)).forward(sink));

        loop {
          tokio::select! {
            Some(msg) = stream.next() => match self.handle_stream_message(msg.map_err(|e| e.into())) {
              Ok(loop_flow) => match loop_flow {
                LoopFlow::Break => break,
                LoopFlow::Continue => continue,
              },
              Err(e) => {
                eprintln!("handle stream message error: {e}. will disconnect");
                break;
              }
            },
            _ = ping_check_interval.tick() => {
              let last_ping_elapsed = self.last_ping.elapsed();
              if last_ping_elapsed > option.close_after {
                warn!("{} since last ping, will exit", last_ping_elapsed.as_secs());
                break;
              }
            }
          }
        }
      }

      if let Some(emitter) = &self.emitter {
        let _ = emitter
          .send(Message::Close(None))
          .await
          .inspect_err(|e| error!("send close fail: {}", e));
      }
      self.emitter = None;
      tokio::time::sleep(option.retry_interval).await;

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

  pub async fn close(&self) -> Result<()> {
    let emitter = self.emitter.clone().ok_or(anyhow!("no emitter know"))?;
    emitter.send(Message::Close(None)).await?;
    Ok(())
  }
}

pub trait WSTransform {
  fn to_str<T: Serialize + Display>(&self, data: T) -> Result<String> {
    Ok(serde_json::to_string::<T>(&data)?)
  }

  fn from_str<O: serde::de::DeserializeOwned>(&self, data: &str) -> Result<O> {
    Ok(serde_json::from_str::<O>(data)?)
  }

  fn from_slice<O: serde::de::DeserializeOwned>(&self, data: &[u8]) -> Result<O> {
    Ok(serde_json::from_slice::<O>(data)?)
  }

  fn on_open<O: serde::de::DeserializeOwned + Display>(&self) -> Result<O> {
    Ok(serde_json::from_value::<O>(serde_json::Value::Null)?)
  }

  fn on_close<O: serde::de::DeserializeOwned + Display>(&self) -> Result<O> {
    Ok(serde_json::from_value::<O>(serde_json::Value::Null)?)
  }
}
