use std::time::Duration;

use mystar_core::{
  logger::init_log,
  net::{run, WSClientCtrl},
};
use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::tungstenite::Message;

use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_log("./log/net-{}.log")?;

  let (tx_cmd, rx_cmd) = unbounded_channel::<WSClientCtrl>();
  let (tx_msg, mut rx_msg) = unbounded_channel::<Message>();

  tokio::spawn(run((tx_msg, rx_cmd)));

  tokio::spawn(async move {
    while let Some(msg) = rx_msg.recv().await {
      info!("message: {msg:?}")
    }
  });

  tokio::time::sleep(Duration::from_secs(30)).await;
  tx_cmd.send(WSClientCtrl::Exit).unwrap();

  tokio::time::sleep(Duration::from_secs(3)).await;

  Ok(())
}
