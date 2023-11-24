use std::{
  net::TcpStream,
  time::{Duration, Instant},
};

use anyhow::{anyhow, Result};

use erased_serde::{Deserializer, Serialize};
use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use tokio::sync::{
  broadcast,
  mpsc::{self, Sender},
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use log::{error, info, warn};

pub struct WSOption {
  url: String,
  close_after: Duration,
  retry_interval: Duration,
  retry_max_count: Option<u32>,
}

pub struct WSClient {
  transformer: Option<Box<dyn WSTransform>>,
  emitter: Option<Sender<Message>>,
  handler: broadcast::Sender<EventMessage>,
  last_ping: Instant,
}

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

enum LoopFlow {
  Break,
  Continue,
}

#[derive(Clone, Debug)]
enum EventMessage {
  Text(String),
  Binary(Vec<u8>),
}

impl WSClient {
  pub fn new() -> Self {
    let (handler, _) = broadcast::channel::<EventMessage>(128);
    Self {
      transformer: None,
      emitter: None,
      handler,
      last_ping: Instant::now(),
    }
  }
  pub fn transform(&mut self, t: Box<dyn WSTransform>) {
    self.transformer = Some(t);
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

  pub async fn send(&self, data: Box<dyn Serialize>) -> Result<()> {
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

  pub async fn close(&self) -> Result<()> {
    let emitter = self.emitter.clone().ok_or(anyhow!("no emitter know"))?;
    emitter.send(Message::Close(None)).await?;
    Ok(())
  }

  pub async fn recv<F, T>(&self, cb: F) -> Result<()>
  where
    T: DeserializeOwned,
    F: Fn(T) -> (),
  {
    let mut rcv = self.handler.subscribe();
    while let Ok(msg) = rcv.recv().await {
      match msg {
        EventMessage::Text(text) => {
          if let Some(ts) = &self.transformer {
            let der = &mut ts.from_str(&text);
            let val = erased_serde::deserialize::<T>(der)?;
            cb(val);
          } else {
            cb(serde_json::from_str::<T>(&text)?);
          }
        }
        EventMessage::Binary(bin) => {
          if let Some(ts) = &self.transformer {
            let der = &mut ts.from_slice(&bin);
            let val = erased_serde::deserialize::<T>(der)?;
            cb(val);
          } else {
            cb(serde_json::from_slice::<T>(&bin)?);
          }
        }
      };
    }
    error!("receive error");
    Ok(())
  }
}

pub trait WSTransform {
  fn to_str(&self, data: Box<dyn Serialize>) -> Result<String>;
  // fn to_buffer(&self, data: Box<dyn Serialize>) -> Vec<u8>;
  fn from_str(&self, data: &str) -> Box<dyn Deserializer>;
  fn from_slice(&self, data: &[u8]) -> Box<dyn Deserializer>;
}
