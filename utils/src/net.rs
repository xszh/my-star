use std::{clone, fmt::Debug, sync::Arc, time::Duration};

use futures_util::{stream::SplitStream, StreamExt};

use parking_lot::Mutex;
use tokio::{
  io::{AsyncRead, AsyncWrite},
  sync::{
    broadcast,
    mpsc::{UnboundedReceiver, UnboundedSender},
  },
  sync::{broadcast::Sender, mpsc::unbounded_channel},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{
  connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream,
  WebSocketStream,
};

use erased_serde::Serialize;

use log::{error, info, warn};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WSClientHeader {
  pub token: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WSClientData<T: serde::Serialize> {
  command: u32,
  header: WSClientHeader,
  data: T,
}

impl<T: serde::Serialize> WSClientData<T> {
  pub fn new<F>(command: u32, data: T, gen_header: F) -> Self
  where
    F: Fn() -> WSClientHeader,
  {
    WSClientData {
      command,
      header: gen_header(),
      data,
    }
  }

  pub fn to_json(&self) -> anyhow::Result<String> {
    Ok(serde_json::to_string::<WSClientData<T>>(&self)?)
  }
}

pub enum WSClientCtrl {
  Exit,
  Send((u32, Box<dyn Serialize + Sync + Send>)),
  SendText(String),
  SendBinary(Vec<u8>),
}

#[derive(Clone, Debug)]
pub enum WSMessage {
  Open(),
  Close(),
  Text(String),
  Binary(Vec<u8>),
  JSON(serde_json::Value),
}

fn handle_msg(msg: Message, tx: Sender<WSMessage>) -> anyhow::Result<()> {
  match msg {
    Message::Text(text) => {
      if let Ok(val) = serde_json::from_str(&text) {
        tx.send(WSMessage::JSON(val))?;
      } else {
        tx.send(WSMessage::Text(text))?;
      }
    }
    Message::Binary(bin) => {
      if let Ok(val) = serde_json::from_slice(&bin) {
        tx.send(WSMessage::JSON(val))?;
      } else {
        tx.send(WSMessage::Binary(bin))?;
      }
    }
    Message::Ping(_) => info!("on ping"),
    Message::Pong(_) => info!("on pong"),
    Message::Close(_) => {
      tx.send(WSMessage::Close())?;
    }
    Message::Frame(_) => info!("on frame"),
  };
  Ok(())
}

fn handle_ctrl<F>(
  ctrl: WSClientCtrl,
  tx_msg: UnboundedSender<Message>,
  gen_header: F,
) -> anyhow::Result<bool>
where
  F: Fn() -> WSClientHeader + Copy,
{
  match ctrl {
    WSClientCtrl::Exit => {
      info!("ws ctrl: exit");
      return Ok(true);
    }
    WSClientCtrl::SendText(str) => {
      tx_msg.send(Message::Text(str))?;
    }
    WSClientCtrl::SendBinary(bin) => {
      tx_msg.send(Message::Binary(bin))?;
    }
    WSClientCtrl::Send((command, data)) => {
      match WSClientData::new(command, data, gen_header).to_json() {
        Ok(str) => {
          tx_msg.send(Message::Text(str))?;
        }
        Err(e) => {
          error!("toJSON failed {}", e);
        }
      }
    }
  };
  return Ok(false);
}

async fn connected<S, F>(
  mut stream: SplitStream<WebSocketStream<MaybeTlsStream<S>>>,
  gen_header: F,
  in_tx: UnboundedSender<Message>,
  out_tx: Sender<WSMessage>,
  ctrl_rx: &mut UnboundedReceiver<WSClientCtrl>,
) -> anyhow::Result<bool>
where
  S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
  MaybeTlsStream<S>: Unpin,
  F: Fn() -> WSClientHeader + Copy,
{
  let mut exit = false;

  let open_str = WSClientData::new(0, (), gen_header).to_json()?;
  info!("open cmd: {open_str}");
  in_tx.send(Message::Text(open_str))?;

  loop {
    tokio::select! {
      Some(msg) = stream.next() => {
        match msg {
          Ok(msg) => {
            handle_msg(msg, out_tx.clone())?;
            continue;
          },
          Err(e) => {
            error!("receive msg fail: {e}");
            break;
          }
        }
      },
      Some(ctrl) = ctrl_rx.recv() => {
        if handle_ctrl(ctrl, in_tx.clone(), gen_header)? {
          exit = true;
          break;
        }
      },
      else => {
        break;
      }
    };
  }
  in_tx.send(Message::Close(None))?;
  Ok(exit)
}

pub async fn run<R, F>(
  request: R,
  mut rx_cmd: UnboundedReceiver<WSClientCtrl>,
  tx_msg: Sender<WSMessage>,
  gen_header: F,
  retry_timeout: Duration,
  max_retry_count: Option<u32>,
) -> anyhow::Result<()>
where
  R: IntoClientRequest + Unpin + Clone,
  F: Fn() -> WSClientHeader + Copy,
{
  let mut retry_count = 0;

  loop {
    if let Ok((ws_stream, _)) = connect_async(request.clone()).await {
      let (sender, stream) = ws_stream.split();
      let (ws_tx, pipe_rx) = unbounded_channel::<Message>();
      let rx_stream: UnboundedReceiverStream<Message> = pipe_rx.into();
      tokio::spawn(rx_stream.map(|m| Ok(m)).forward(sender));

      let in_tx = ws_tx.clone();
      match connected(stream, gen_header, in_tx, tx_msg.clone(), &mut rx_cmd).await {
        Ok(exit) => {
          if exit {
            return Ok(());
          } else {
            warn!("connect closed, will try connect again");
          }
        }
        Err(e) => {
          error!("fatal error: {e}");
          ws_tx.send(Message::Close(None))?;
          return Err(e);
        }
      }
    };

    tokio::time::sleep(retry_timeout).await;

    retry_count += 1;
    if let Some(max_retry_count) = max_retry_count {
      if retry_count >= max_retry_count {
        error!("exceed max retry count: {}", max_retry_count);
        return Ok(());
      }
    }
  }
}

pub trait GenHader {
  fn gen_header(&self) -> WSClientHeader;
}

pub struct WSClient<T: GenHader> {
  pub request: String,
  pub retry_interval: Duration,
  pub retry_max_count: Option<u32>,
  gen_header: T,
  tx_cmd: UnboundedSender<WSClientCtrl>,
  rx_cmd: Arc<Mutex<UnboundedReceiver<WSClientCtrl>>>,
  tx_msg: broadcast::Sender<WSMessage>,
  rx_msg: broadcast::Receiver<WSMessage>,
}
