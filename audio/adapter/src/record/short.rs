use std::{
  default,
  sync::{Arc, LazyLock},
  time::{Duration, Instant},
};

use crate::{resample, utils::UBChannel};
use anyhow::{anyhow, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, Sample, SampleFormat, SizedSample, SupportedStreamConfig,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use rubato::Resampler;

#[derive(Debug)]
pub enum ShortRecordChannel {
  Start,
  Stop,
  Close,
}

type AM<T> = Arc<Mutex<T>>;
type ARW<T> = Arc<RwLock<T>>;

struct ShortRecord {
  buffer: ARW<Vec<f32>>,
  ch_cmd: UBChannel<ShortRecordChannel>,
  ch_data: UBChannel<Vec<i16>>,
  recording: AM<bool>,
  capturing: AM<bool>,
}

impl ShortRecord {
  pub fn set_capturing(&self, value: bool) {
    *self.capturing.lock() = value;
  }

  pub fn set_recording(&self, value: bool) {
    *self.recording.lock() = value;
  }
}

fn get_recorder() -> &'static LazyLock<ShortRecord> {
  static RECORDER: LazyLock<ShortRecord> = LazyLock::new(|| ShortRecord {
    buffer: Arc::new(RwLock::new(vec![])),
    ch_cmd: UBChannel::new(),
    ch_data: UBChannel::new(),
    recording: Arc::new(Mutex::new(false)),
    capturing: Arc::new(Mutex::new(false)),
  });
  &RECORDER
}

pub fn is_recording() -> bool {
  *get_recorder().recording.lock()
}

pub fn is_capturing() -> bool {
  *get_recorder().capturing.lock()
}

pub fn capturing() -> Result<()> {
  let host = cpal::default_host();
  let device = host.default_input_device().ok_or(anyhow!(""))?;
  let raw_config = device.default_input_config()?;
  let config = raw_config.clone();

  let stream = match config.sample_format() {
    SampleFormat::I16 => run::<i16>(&device, &config),
    SampleFormat::I32 => run::<i32>(&device, &config),
    SampleFormat::I64 => run::<i64>(&device, &config),
    SampleFormat::U8 => run::<u8>(&device, &config),
    SampleFormat::U16 => run::<u16>(&device, &config),
    SampleFormat::U32 => run::<u32>(&device, &config),
    SampleFormat::U64 => run::<u64>(&device, &config),
    SampleFormat::F32 => run::<f32>(&device, &config),
    SampleFormat::F64 => run::<f64>(&device, &config),
    s => Err(anyhow!("unsupported sample format: {}", s)),
  }?;

  get_recorder().set_capturing(true);
  stream.play()?;

  loop {
    match get_recorder().ch_cmd.try_recv() {
      Ok(msg) => {
        println!("ShortRecordChannel::{:?}", msg);
        match msg {
          ShortRecordChannel::Start => {
            get_recorder().set_recording(true);
            get_recorder().buffer.write().clear();
            continue;
          }
          ShortRecordChannel::Stop => {
            get_recorder().set_recording(false);

            let buf = get_recorder()
              .buffer
              .write()
              .drain(..)
              .collect::<Vec<f32>>();

            let config = raw_config.clone();
            std::thread::spawn(move || -> Result<()> {
              get_recorder()
                .ch_data
                .send(match finalized_data(&config, &buf) {
                  Ok(data) => data,
                  Err(e) => {
                    eprintln!("finalize data fail: {}", e);
                    Vec::<i16>::new()
                  }
                });

              get_recorder().set_recording(false);
              Ok(())
            });
            continue;
          }
          ShortRecordChannel::Close => {
            break;
          }
        }
      }
      Err(e) => {
        if e.is_empty() {
          continue;
        } else {
          eprintln!("try recv error: {e}");
          break;
        }
      }
    }
  }

  drop(stream);
  get_recorder().set_capturing(false);
  Ok(())
}

fn finalized_data(config: &SupportedStreamConfig, buf: &[f32]) -> Result<Vec<i16>> {
  let mut resampler =
    resample::resample::<f32>(config.sample_rate().0 as f64, 16000f64, buf.len())?;
  let mut out = resampler.output_buffer_allocate(true);
  resampler.process_into_buffer(&vec![buf], &mut out, None)?;
  let out = out
    .get(0)
    .ok_or(anyhow!("get first channel fail"))?
    .to_vec();

  Ok(
    out
      .into_iter()
      .map(|item| i16::from_sample(item))
      .collect::<Vec<i16>>(),
  )
}

pub fn open() -> Result<()> {
  // let ctrl = get_recorder().ch_cmd.r;
  std::thread::spawn(move || {
    capturing().expect("??");
  });

  Ok(())
}

pub fn close() -> Result<()> {
  get_recorder().ch_cmd.send(ShortRecordChannel::Close);
  Ok(())
}

pub fn start() -> Result<()> {
  if is_recording() {
    return Ok(());
  }
  println!("short::start");
  get_recorder().ch_cmd.send(ShortRecordChannel::Start);
  Ok(())
}

pub fn stop() -> Result<Vec<i16>> {
  if !is_recording() {
    return Ok(vec![]);
  }
  println!("short::stop");
  get_recorder().ch_cmd.send(ShortRecordChannel::Stop);

  let final_data = get_recorder()
    .ch_data
    .recv_timeout(Duration::from_secs(5))?;

  Ok(final_data)
}

fn run<F>(device: &cpal::Device, config: &cpal::SupportedStreamConfig) -> Result<cpal::Stream>
where
  F: SizedSample,
  f32: FromSample<F>,
{
  let channel = config.clone().channels();
  Ok(device.build_input_stream(
    &config.clone().into(),
    move |data: &[F], _: &_| {
      if is_recording() {
        let rec = get_recorder();
        let mut buf = rec.buffer.write();
        write_data(data, &mut buf, channel);
      }
    },
    |e| println!("input stream fail: {}", e),
    None,
  )?)
}

fn write_data<F>(data: &[F], buffer: &mut Vec<f32>, channel: u16)
where
  F: SizedSample,
  f32: FromSample<F>,
{
  let mut idx: usize = 0;
  let mut c_idx: u16 = 1;
  let mut d: f32 = 0f32;
  while idx < data.len() {
    let x = f32::from_sample(data[idx]) / (channel as f32);
    d += x;
    if c_idx == channel {
      buffer.push(d);
      d = 0f32;
      c_idx = 1;
    } else {
      c_idx += 1;
    }
    idx += 1;
  }
}

#[test]
fn test() {
  open().expect("open");
  std::thread::sleep(Duration::from_secs(1));
  start().expect("start");
  std::thread::sleep(Duration::from_secs(3));
  let d = stop().expect("stop");
  println!("d len: {}", d.len());
  std::thread::sleep(Duration::from_secs(1));
  close().expect("close");
}
