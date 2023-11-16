use std::{collections::HashMap, default, sync::LazyLock};

use crate::audio::{start_record, stop_record};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub(crate) async fn start_asr(app: tauri::AppHandle) -> Result<()> {
  start_record(app).await
}

#[derive(Serialize, Deserialize)]
struct ASRResult {
  task_id: String,
  result: String,
  status: u32,
  message: String,
}

pub(crate) async fn stop_asr<S1, S2>(app: tauri::AppHandle, token: S1, app_id: S2) -> Result<String>
where
  S1: AsRef<str>,
  S2: AsRef<str>,
{
  let buffer = stop_record(app)?;

  if buffer.is_empty() {
    println!("asr buffer is empty");
    return Ok("".into());
  }

  let client = reqwest::Client::new();
  let mut params: HashMap<&str, &str> = Default::default();
  params.insert("appkey", app_id.as_ref());
  params.insert("format", "pcm");
  params.insert("sample_rate", "16000");
  params.insert("enable_punctuation_prediction", "false");
  params.insert("enable_inverse_text_normalization", "true");
  let res = client
    .post("https://nls-gateway-cn-shanghai.aliyuncs.com/stream/v1/asr")
    .header("X-NLS-Token", token.as_ref())
    .header("Content-type", "application/octet-stream")
    .header("Content-Length", buffer.len())
    .query(&params)
    .body(buffer)
    .send()
    .await?;

  let res: ASRResult = serde_json::from_str(res.text().await?.as_str())?;

  match res.status {
    20000000 => Ok(res.result),
    _ => Err(anyhow::anyhow!(res.message)),
  }
}
