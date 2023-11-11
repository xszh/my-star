use anyhow::{anyhow, Ok, Result};
use std::{
  any,
  sync::{Arc, LazyLock},
};
use tauri::Manager;

use adapter::record::ShortRecord;
use parking_lot::{Mutex, RwLock};

static RECORDING: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

pub(crate) fn is_recording() -> bool {
  *RECORDING.lock()
}

fn set_recording(value: bool, app: tauri::AppHandle) -> anyhow::Result<()> {
  app.emit_all("recording", value)?;
  *RECORDING.lock() = value;
  Ok(())
}

pub(crate) fn get_recorder() -> &'static LazyLock<Arc<RwLock<Option<ShortRecord>>>> {
  static RECORDER: LazyLock<Arc<RwLock<Option<ShortRecord>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));
  &RECORDER
}

async fn _start_record() -> anyhow::Result<()> {
  get_recorder()
    .read()
    .as_ref()
    .ok_or(anyhow!("no recorder while start record"))?
    .start()?;
  Ok(())
}

pub(crate) async fn start_record(app: tauri::AppHandle) -> anyhow::Result<()> {
  if is_recording() {
    return Ok(());
  }
  set_recording(true, app)?;

  if get_recorder().read().is_none() {
    *get_recorder().write() = Some(ShortRecord::new()?);
  }

  tokio::spawn(_start_record());

  Ok(())
}

pub(crate) fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>> {
  if !is_recording() {
    return Ok(vec![]);
  }
  set_recording(false, app)?;

  let data = get_recorder()
    .read()
    .as_ref()
    .ok_or(anyhow::anyhow!("no recorder while stop recording"))?
    .stop()?;

  Ok(data.iter().flat_map(|d| d.to_le_bytes()).collect())
}
