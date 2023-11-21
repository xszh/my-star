use std::{env, time::Duration};

use futures_util::{SinkExt, StreamExt};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use mystar_net::{run, WSClientCtrl};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let (tx_cmd, rx_cmd) = tokio::sync::mpsc::unbounded_channel::<WSClientCtrl>();
  let (tx_msg, mut rx_msg) = tokio::sync::mpsc::unbounded_channel::<Message>();

  tokio::spawn(run((tx_msg, rx_cmd)));

  tokio::spawn(async move {
    while let Some(msg) = rx_msg.recv().await {
      println!("message: {msg:?}")
    }
  });

  tokio::time::sleep(Duration::from_secs(5)).await;
  tx_cmd.send(WSClientCtrl::Exit).unwrap();
  
  tokio::time::sleep(Duration::from_secs(3)).await;

  Ok(())
}
