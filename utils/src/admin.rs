use std::{
  collections::{HashMap, HashSet},
  fmt::Display,
  time::Duration,
};

use crate::core::ws::{WSCell, WSClient, WSOption};

use anyhow::{anyhow, Result};

use log::{error, info, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Map;

#[derive(Clone)]
pub struct AdminWSContext {
  pub token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminWSHeader {
  token: String,
}

impl AdminWSContext {
  fn gen_header(&self) -> AdminWSHeader {
    AdminWSHeader {
      token: self.token.clone(),
    }
  }
}

#[repr(u32)]
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AdminWSCell {
  Open {} = 0,
  Close {} = 1,
  Test { data: String } = 2,
}

impl WSCell<AdminWSContext> for AdminWSCell {
  fn to_string(&self, context: &Option<AdminWSContext>) -> Result<String>
  where
    Self: Serialize,
  {
    let mut send_data = Map::new();
    send_data.insert("data".into(), serde_json::to_value(self)?);

    if let Some(context) = context {
      send_data.insert("header".into(), serde_json::to_value(context.gen_header())?);
    }

    Ok(serde_json::to_string(&send_data)?)
  }
  fn open_cell(_: &Option<AdminWSContext>) -> Result<Self>
  where
    Self: DeserializeOwned,
  {
    Ok(AdminWSCell::Open {})
  }

  fn close_cell(_: &Option<AdminWSContext>) -> Result<Self>
  where
    Self: DeserializeOwned,
  {
    Ok(AdminWSCell::Close {})
  }
}

impl Display for AdminWSCell {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AdminWSCell::Open {} => write!(f, "open()"),
      AdminWSCell::Close {} => write!(f, "close()"),
      AdminWSCell::Test { data } => write!(f, "test({data}: String)"),
    }
  }
}

pub struct AdminWSClient<'s> {
  ws_client: WSClient<AdminWSContext>,
  event_map: HashMap<AdminWSCell, Vec<Box<dyn Send + Sync + Fn(&Self, AdminWSCell) -> () + 's>>>,
}

impl<'s> AdminWSClient<'s> {
  pub fn init() -> Self {
    Self {
      ws_client: WSClient::<AdminWSContext>::new(),
      event_map: HashMap::new(),
    }
  }
  pub fn emit(&self, cell: AdminWSCell) -> &Self {
    let client = self.ws_client.clone();
    tokio::spawn(async move { client.send(cell).await });
    self
  }
  pub fn on<'a, F>(&'a mut self, cell: AdminWSCell, cb: F) -> &'a mut Self
  where
    F: Send + Sync + Fn(&Self, AdminWSCell) -> () + 's,
  {
    self
      .event_map
      .entry(cell)
      .or_insert(vec![])
      .push(Box::new(cb));
    self
  }
  pub fn context<'a>(&'a mut self, c: AdminWSContext) -> &'a mut Self {
    self.ws_client.context(c);
    self
  }
  pub async fn run(&mut self) {
    let option = WSOption {
      url: "wss://www.miemie.tech/mystar/ws/".into(),
      retry_interval: Duration::from_secs(5),
      close_after: Duration::from_secs(20),
      retry_max_count: None,
    };

    let ee = &mut self.ws_client.subscribe();
    tokio::select! {
      _ = async {
        while let Ok(msg) = self.ws_client.inner_recv::<AdminWSCell>(ee).await {
          if let Some(cbs) = self.event_map.get(&msg) {
            cbs.iter().for_each(|cb| {
              cb(&self, msg.clone());
            })
          }
        }
      } => {},
      Err(e) = self.ws_client.connect(&option) => {
        error!("disconnect: {e}");
      },
    }
  }
}
