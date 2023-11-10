use anyhow::{Ok, Result};
use tauri::Manager;
use std::sync::{Arc, LazyLock};

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

pub(crate) fn start_record(app: tauri::AppHandle) -> anyhow::Result<()> {
  if is_recording() {
    return Ok(());
  }
  set_recording(true, app)?;

  get_recorder()
    .write()
    .get_or_insert(ShortRecord::new()?)
    .start()?;

  Ok(())
}

pub(crate) fn stop_record(app: tauri::AppHandle) -> Result<Vec<u8>> {
  if !is_recording() {
    return Ok(vec![]);
  }
  set_recording(false, app)?;

  let data = get_recorder()
    .write()
    .get_or_insert(ShortRecord::new()?)
    .stop()?;

  Ok(data.iter().flat_map(|d| d.to_le_bytes()).collect())
}