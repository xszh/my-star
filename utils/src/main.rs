use mystar::admin::{AdminWSCell, AdminWSClient, AdminWSContext};
use mystar::core::logger::init_log;

use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_log("./log/net-{}.log")?;

  let mut client = AdminWSClient::init();

  client
    .context(AdminWSContext {
      token: "test_token".into(),
    })
    .on(AdminWSCell::Open {}, |client, cell| {
      info!("receive open");
      client.emit(AdminWSCell::Open {});
    })
    .on(AdminWSCell::Close {}, |client, cell| {
      info!("receive close");
    });

  client.run().await;

  Ok(())
}
