use anyhow::{anyhow, Ok, Result};
use std::{
  any,
  ops::Deref,
  sync::{Arc, LazyLock},
};
use tauri::Manager;

use adapter::record::ShortRecord;
use parking_lot::{Mutex, RwLock};

pub(crate) fn is_capturing() -> bool {
  get_recorder()
    .read()
    .as_ref()
    .map_or(false, |r| r.is_capturing())
}

pub(crate) fn is_recording() -> bool {
  get_recorder()
    .read()
    .as_ref()
    .map_or(false, |r| r.is_buffering())
}

fn set_capturing(value: bool, app: tauri::AppHandle) -> anyhow::Result<()> {
  app.emit_all("capturing", value)?;
  Ok(())
}

fn set_recording(value: bool, app: tauri::AppHandle) -> anyhow::Result<()> {
  app.emit_all("recording", value)?;
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

pub(crate) async fn audio_open(app: tauri::AppHandle) -> anyhow::Result<()> {
  if is_capturing() {
    return Ok(());
  }

  if get_recorder().read().is_none() {
    *get_recorder().write() = Some(ShortRecord::new()?);
  }

  get_recorder()
    .read()
    .as_ref()
    .ok_or(anyhow!("no recorder while start record"))?
    .open()?;

  set_capturing(true, app)?;
  Ok(())
}

pub(crate) async fn audio_close(app: tauri::AppHandle) -> anyhow::Result<()> {
  if !is_capturing() {
    return Ok(());
  }

  get_recorder()
    .read()
    .as_ref()
    .take()
    .ok_or(anyhow!("no recorder while close record"))?
    .close()?;

  set_capturing(false, app);
  Ok(())
}

pub(crate) async fn start_record(app: tauri::AppHandle) -> anyhow::Result<()> {
  if !is_capturing() {
    return Err(anyhow!("not capture"));
  }

  if is_recording() {
    return Ok(());
  }

  set_recording(true, app)?;

  tokio::spawn(_start_record());

  Ok(())
}

pub(crate) fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>> {
  if !is_capturing() {
    return Err(anyhow!("not capture"));
  }
  if !is_capturing() {
    return Ok(vec![]);
  }

  let data = get_recorder()
    .read()
    .as_ref()
    .ok_or(anyhow::anyhow!("no recorder while stop recording"))?
    .stop()?;

  set_recording(false, app)?;

  Ok(data.iter().flat_map(|d| d.to_le_bytes()).collect())
}
