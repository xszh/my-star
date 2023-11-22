use std::time::Duration;

use futures_util::StreamExt;

use tokio::{
  io::{AsyncRead, AsyncWrite},
  sync::mpsc::UnboundedReceiver,
  sync::{broadcast::Sender, mpsc::unbounded_channel},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{
  connect_async, tungstenite::client::IntoClientRequest, tungstenite::Message, MaybeTlsStream,
  WebSocketStream,
};

use log::{error, info, warn};

pub enum WSClientCtrl {
  Exit,
  SendText(String),
  SendBinary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum WSMessage {
  Open(),
  Close(),
  Text(String),
  Binary(Vec<u8>)
}

fn handle_msg(msg: Message, tx: Sender<WSMessage>) -> anyhow::Result<()> {
  match msg {
    Message::Text(text) => {
      tx.send(WSMessage::Text(text))?;
    }
    Message::Binary(bin) => {
      tx.send(WSMessage::Binary(bin))?;
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

async fn connected<S>(
  ws_stream: WebSocketStream<MaybeTlsStream<S>>,
  tx: Sender<WSMessage>,
  rx: &mut UnboundedReceiver<WSClientCtrl>,
) -> bool
where
  S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
  MaybeTlsStream<S>: Unpin,
{
  let mut exit = false;

  let (sender, mut stream) = ws_stream.split();
  let (_tx, _rx) = unbounded_channel::<Message>();
  let rx_stream: UnboundedReceiverStream<Message> = _rx.into();

  tokio::spawn(rx_stream.map(|r| Ok(r)).forward(sender));

  // _tx.send(message)

  loop {
    tokio::select! {
      Some(msg) = stream.next() => {
        match msg {
          Ok(msg) => {
            if let Err(e) = handle_msg(msg, tx.clone()) {
              error!("send fail: {}", e);
              break;
            } else {
              continue;
            }
          },
          Err(e) => {
            error!("receive msg fail: {e}");
            break;
          }
        }
      },
      Some(ctrl) = rx.recv() => match ctrl {
        WSClientCtrl::Exit => {
          exit = true;
          info!("ws ctrl: exit");
          break;
        },
        WSClientCtrl::SendText(str) => {
          if let Err(e) = _tx.send(Message::Text(str)) {
            exit = true;
            error!("ws send text fail: {e}");
            break;
          }
        },
        WSClientCtrl::SendBinary(bin) => {
          if let Err(e) = _tx.send(Message::Binary(bin)) {
            exit = true;
            error!("ws send binary fail: {e}");
            break;
          }
        }
      },
      else => {
        break;
      }
    };
  }
  if let Err(e) = _tx.send(Message::Close(None)) {
    error!("ws send binary fail: {e}");
  }
  exit
}

pub async fn run<R>(
  request: R,
  mut rx_cmd: UnboundedReceiver<WSClientCtrl>,
  tx_msg: Sender<WSMessage>,
  retry_timeout: Duration,
  max_retry_count: Option<u32>,
) -> ()
where
  R: IntoClientRequest + Unpin + Clone,
{
  let mut retry_count = 0;

  loop {
    if let Ok((ws_stream, _)) = connect_async(request.clone()).await {
      if connected(ws_stream, tx_msg.clone(), &mut rx_cmd).await {
        break;
      } else {
        warn!("connect closed, will try connect again");
      }
    }

    tokio::time::sleep(retry_timeout).await;

    retry_count += 1;
    if let Some(max_retry_count) = max_retry_count {
      if retry_count >= max_retry_count {
        error!("exceed max retry count: {}", max_retry_count);
        break;
      }
    }
  }
}
