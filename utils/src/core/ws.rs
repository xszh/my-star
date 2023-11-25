use core::slice;
use std::{
  fmt::Display,
  sync::Arc,
  time::{Duration, Instant},
};

use anyhow::{anyhow, Result};

use futures_util::StreamExt;
use parking_lot::RwLock;
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
pub struct WSClient<C> {
  context: Option<C>,
  emitter: Arc<RwLock<Option<Sender<Message>>>>,
  handler: broadcast::Sender<EventMessage>,
}

enum LoopFlow {
  Break,
  Continue,
  Ping,
}

#[derive(Clone, Debug)]
pub enum EventMessage {
  Open(),
  Close(),
  Text(String),
  Binary(Vec<u8>),
}

impl<C> WSClient<C> {
  pub fn new() -> Self {
    let (handler, _) = broadcast::channel::<EventMessage>(128);
    Self {
      context: None,
      emitter: Arc::new(RwLock::new(None)),
      handler,
    }
  }

  pub fn subscribe(&self) -> broadcast::Receiver<EventMessage> {
    self.handler.subscribe()
  }

  fn handle_stream_message(&self, message: Result<Message>) -> Result<LoopFlow> {
    if let Err(e) = message {
      eprintln!("get stream message: {e}");
      return Ok(LoopFlow::Break);
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
      Message::Ping(_) => Ok(LoopFlow::Ping),
      msg => {
        info!("receive message: {msg:?}");
        Ok(LoopFlow::Continue)
      }
    }
  }

  pub async fn connect(&self, option: &WSOption) -> anyhow::Result<()> {
    let mut retry_cnt = 0;

    loop {
      *self.emitter.write() = None;
      let mut ping_check_interval = tokio::time::interval(Duration::from_secs(1));

      info!("start connect to {}", option.url);
      if let Ok((ws_stream, _)) = connect_async(option.url.clone()).await {
        info!("connect success");
        let mut last_ping = Instant::now();

        let (sink, mut stream) = ws_stream.split();
        let (emitter, forwarder) = mpsc::channel::<Message>(128);
        *self.emitter.write() = Some(emitter);
        let forwarder_stream: ReceiverStream<Message> = forwarder.into();
        tokio::spawn(forwarder_stream.map(|m| Ok(m)).forward(sink));

        info!("send open");
        self.handler.send(EventMessage::Open())?;
        info!("start message recv loog");
        loop {
          tokio::select! {
            Some(msg) = stream.next() => match self.handle_stream_message(msg.map_err(|e| e.into())) {
              Ok(loop_flow) => match loop_flow {
                LoopFlow::Break => break,
                LoopFlow::Continue => continue,
                LoopFlow::Ping => {
                  last_ping = Instant::now();
                  continue;
                }
              },
              Err(e) => {
                eprintln!("handle stream message error: {e}. will disconnect");
                break;
              }
            },
            _ = ping_check_interval.tick() => {
              let last_ping_elapsed = last_ping.elapsed();
              if last_ping_elapsed > option.close_after {
                warn!("{} since last ping, will exit", last_ping_elapsed.as_secs());
                break;
              }
            }
          }
        }
      }

      if let Some(emitter) = self.emitter.read().as_ref() {
        let _ = emitter
          .send(Message::Close(None))
          .await
          .inspect_err(|e| error!("send close fail: {}", e));
      }
      *self.emitter.write() = None;
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
    let emitter = self
      .emitter
      .read()
      .clone()
      .ok_or(anyhow!("no emitter know"))?;
    emitter.send(Message::Close(None)).await?;
    Ok(())
  }

  pub fn context(&mut self, t: C) {
    self.context = Some(t);
  }

  pub async fn send<D: WSCell<C>>(&self, data: D) -> Result<()>
  where
    D: Serialize,
  {
    let emitter = self
      .emitter
      .read()
      .clone()
      .ok_or(anyhow!("no emitter know"))?;
    if let Ok(text) = data.to_string(&self.context) {
      info!("send string: {text}");
      emitter.send(Message::Text(text)).await?;
    }
    Ok(())
  }

  pub async fn inner_recv<O>(&self, rcv: &mut broadcast::Receiver<EventMessage>) -> Result<O>
  where
    O: WSCell<C> + DeserializeOwned,
  {
    Ok(match rcv.recv().await? {
      EventMessage::Text(text) => O::from_str(&text, &self.context)?,
      EventMessage::Binary(bin) => O::from_slice(&bin, &self.context)?,
      EventMessage::Open() => O::open_cell(&self.context)?,
      EventMessage::Close() => O::close_cell(&self.context)?,
    })
  }

  pub async fn recv<O>(&self) -> Result<O>
  where
    O: WSCell<C> + DeserializeOwned,
  {
    self.inner_recv::<O>(&mut self.handler.subscribe()).await
  }
}

pub trait WSCell<C> {
  fn to_string(&self, context: &Option<C>) -> Result<String>
  where
    Self: Serialize,
  {
    Ok(serde_json::to_string::<Self>(&self)?)
  }

  fn to_vec(&self, context: &Option<C>) -> Result<Vec<u8>>
  where
    Self: Serialize,
  {
    Ok(serde_json::to_vec::<Self>(&self)?)
  }

  fn from_str(str: &str, context: &Option<C>) -> Result<Self>
  where
    Self: DeserializeOwned,
  {
    Ok(serde_json::from_str::<Self>(str)?)
  }

  fn from_slice(slice: &[u8], context: &Option<C>) -> Result<Self>
  where
    Self: DeserializeOwned,
  {
    Ok(serde_json::from_slice::<Self>(slice)?)
  }

  fn open_cell(context: &Option<C>) -> Result<Self>
  where
    Self: DeserializeOwned;

  fn close_cell(context: &Option<C>) -> Result<Self>
  where
    Self: DeserializeOwned;
}
