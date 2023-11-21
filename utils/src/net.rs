use std::{
  net::TcpStream,
  time::{Duration, Instant},
};

use futures_util::{SinkExt, Stream, StreamExt};

use tokio::{
  io::{AsyncRead, AsyncWrite},
  sync::mpsc::{error, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{
  connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use log::{error, info};

pub enum WSClientCtrl {
  Exit,
}

async fn connected<S>(
  mut ws_stream: WebSocketStream<MaybeTlsStream<S>>,
  tx: UnboundedSender<Message>,
  mut rx: UnboundedReceiver<WSClientCtrl>,
) where
  S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
  MaybeTlsStream<S>: Unpin,
{
  loop {
    tokio::select! {
      Some(msg) = ws_stream.next() => {
        match msg {
          Ok(msg) => {
            tx.send(msg);
            continue;
          },
          Err(e) => {
            error!("receive msg fail: {e}");
            break;
          }
        }
      },
      Some(ctrl) = rx.recv() => match ctrl {
        WSClientCtrl::Exit => {
          info!("ctrl: exit");
          break;
        }
      },
      else => {
        break;
      }
    };
  }
  if let Err(e) = ws_stream.close(None).await {
    error!("close stream fail: {e}");
  }
}

pub async fn run((tx, mut rx): (UnboundedSender<Message>, UnboundedReceiver<WSClientCtrl>)) -> () {
  let addr = "wss://www.miemie.tech/mystar/ws/";

  let ws_stream = connect_async(addr).await;

  if let Ok((ws_stream, _)) = ws_stream {
    connected(ws_stream, tx, rx).await;
  }
}
