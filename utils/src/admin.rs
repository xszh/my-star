use std::{fmt::Display, time::Duration};

use crate::core::ws::{self, WSClient, WSOption, WSTransform};

use anyhow::{anyhow, Result};

use log::{error, info, warn};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone)]
pub struct AdminWSTransform {
  device_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AdminWSHeader {
  device_id: String,
}

impl AdminWSTransform {
  fn gen_header(&self) -> AdminWSHeader {
    AdminWSHeader {
      device_id: self.device_id.clone(),
    }
  }
}

#[repr(u32)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "command")]
pub enum AdminWSCell {
  Open { data: () } = 0,
  Close { data: () } = 1,
  Test { data: String } = 2,
}

impl Display for AdminWSCell {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AdminWSCell::Open { data: _ } => write!(f, "open()"),
      AdminWSCell::Close { data: _ } => write!(f, "close()"),
      AdminWSCell::Test { data } => write!(f, "test({data}: String)"),
    }
  }
}

impl WSTransform for AdminWSTransform {
  fn to_str<T>(&self, data: T) -> Result<String>
  where
    T: Serialize + Display,
  {
    let data_val = data.to_string();
    let mut value = serde_json::to_value(data)?;
    let header = serde_json::to_value(self.gen_header())?;
    let value = value
      .as_object_mut()
      .ok_or(anyhow!("{data_val} not an object"))?;
    value.insert("header".into(), header);
    Ok(serde_json::to_string(value)?)
  }
}

pub type AdminWSClient = WSClient<AdminWSTransform>;

impl AdminWSClient {
  pub async fn init() -> Self {
    let client = Self::new();

    client
  }
  pub fn emit(&self, cell: AdminWSCell) -> &Self {
    let client = (*self).clone();
    tokio::spawn(async move {
      let cell_str = cell.to_string();
      if let Err(e) = client.send(cell).await {
        error!("emit {cell_str} failed: {e}");
      }
    });
    self
  }
  pub async fn run(&mut self) {
    let option = WSOption {
      url: "wss://www.miemie.tech/mystar/ws/".into(),
      retry_interval: Duration::from_secs(5),
      close_after: Duration::from_secs(20),
      retry_max_count: None,
    };
    if let Err(e) = self.connect(&option).await {
      error!("disconnect: {e}");
    }
  }
}
