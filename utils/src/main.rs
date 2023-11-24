use std::time::Duration;

use mystar::{
  core::logger::init_log,
  // net::{run, WSClientCtrl, WSClientHeader, WSMessage},
  impl::admin;
};
use tokio::sync::broadcast;
use tokio::sync::mpsc::unbounded_channel;

use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_log("./log/net-{}.log")?;

  let (tx_cmd, rx_cmd) = unbounded_channel::<WSClientCtrl>();
  let (tx_msg, mut rx_msg) = broadcast::channel::<WSMessage>(16);

  tokio::spawn(run(
    "wss://www.miemie.tech/mystar/ws/",
    rx_cmd,
    tx_msg,
    || {
      return WSClientHeader {
        token: "test_main".into(),
      };
    },
    Duration::from_secs(1),
    None,
  ));

  tokio::spawn(async move {
    while let Ok(msg) = rx_msg.recv().await {
      info!("message: {msg:?}")
    }
  });

  tokio::time::sleep(Duration::from_secs(60)).await;
  tx_cmd.send(WSClientCtrl::Exit).unwrap();

  tokio::time::sleep(Duration::from_secs(3)).await;

  Ok(())
}
