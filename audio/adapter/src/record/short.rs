use std::{ops::DerefMut, sync::Arc, time::Duration};

use crate::resample;
use anyhow::{anyhow, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, Sample, SampleFormat, SizedSample,
};
use crossbeam::channel::{unbounded, Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use rubato::Resampler;

pub struct ShortRecord {
  device: cpal::Device,
  config: cpal::SupportedStreamConfig,
  buffer: Arc<RwLock<Option<Vec<f32>>>>,
  csx: Arc<Mutex<Sender<ShortRecordChannel>>>,
  crx: Receiver<ShortRecordChannel>,
  dsx: Arc<Mutex<Sender<Vec<f32>>>>,
  drx: Receiver<Vec<f32>>,
  capturing: Arc<RwLock<bool>>,
}

enum ShortRecordChannel {
  Start,
  Stop,
  Close,
}

impl ShortRecord {
  pub fn new() -> Result<Self> {
    let host = cpal::default_host();
    let device = host
      .default_input_device()
      .ok_or(anyhow!("no default input device"))?;

    let config = device.default_input_config()?;

    let (csx, crx) = unbounded::<ShortRecordChannel>();
    let (dsx, drx) = unbounded::<Vec<f32>>();

    Ok(Self {
      device,
      config,
      buffer: Arc::new(RwLock::new(None)),
      csx: Arc::new(Mutex::new(csx)),
      crx,
      dsx: Arc::new(Mutex::new(dsx)),
      drx,
      capturing: Arc::new(RwLock::new(false)),
    })
  }

  pub fn is_capturing(&self) -> bool {
    *self.capturing.read()
  }

  pub fn is_buffering(&self) -> bool {
    self.buffer.read().is_some()
  }

  fn run<F>(&self) -> Result<cpal::Stream>
  where
    F: SizedSample,
    f32: FromSample<F>,
  {
    let channel = self.config.clone().channels();
    let buffer = self.buffer.clone();
    Ok(self.device.build_input_stream(
      &self.config.clone().into(),
      move |data: &[F], _: &_| {
        let mut buffer = buffer.write();
        if let Some(mut buffer) = buffer.as_mut() {
          Self::write_data(data, &mut buffer, channel);
        }
      },
      |e| println!("input stream fail: {}", e),
      None,
    )?)
  }

  pub fn open(&self) -> Result<()> {
    if *self.capturing.read() {
      return Ok(());
    }
    *self.capturing.write() = true;
    let stream = match self.config.sample_format() {
      SampleFormat::I16 => self.run::<i16>(),
      SampleFormat::I32 => self.run::<i32>(),
      SampleFormat::I64 => self.run::<i64>(),
      SampleFormat::U8 => self.run::<u8>(),
      SampleFormat::U16 => self.run::<u16>(),
      SampleFormat::U32 => self.run::<u32>(),
      SampleFormat::U64 => self.run::<u64>(),
      SampleFormat::F32 => self.run::<f32>(),
      SampleFormat::F64 => self.run::<f64>(),
      s => Err(anyhow!("unsupported sample format: {}", s)),
    }?;

    stream.play()?;

    let buffer = self.buffer.clone();
    loop {
      match self.crx.try_recv() {
        Ok(msg) => match msg {
          ShortRecordChannel::Start => {
            *buffer.write() = Some(vec![]);
            continue;
          }
          ShortRecordChannel::Stop => {
            let buf = buffer
              .write()
              .take()
              .ok_or(anyhow!("stop while no buffer"))?;

            self.dsx.lock().send(buf)?;
            continue;
          }
          ShortRecordChannel::Close => {
            break;
          }
        },
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

    *self.capturing.write() = false;
    Ok(())
  }

  pub fn close(&self) -> anyhow::Result<()> {
    if !self.is_capturing() {
      return Err(anyhow!("not open yet"));
    }
    Ok(self.csx.lock().send(ShortRecordChannel::Close)?)
  }

  pub fn start(&self) -> anyhow::Result<()> {
    if self.is_buffering() {
      return Ok(());
    }
    Ok(self.csx.lock().send(ShortRecordChannel::Start)?)
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

  pub fn stop(&self) -> Result<Vec<i16>> {
    if !self.is_buffering() {
      return Err(anyhow!("not start yet"));
    }

    self.csx.clone().lock().send(ShortRecordChannel::Stop)?;

    let raw = self.drx.recv_timeout(Duration::from_secs(5))?;
    let mut resampler =
      resample::resample::<f32>(self.config.sample_rate().0 as f64, 16000f64, raw.len())?;
    let mut out = resampler.output_buffer_allocate(true);
    resampler.process_into_buffer(&vec![raw], &mut out, None)?;

    let out = out
      .get(0)
      .ok_or(anyhow!("get first channel fail"))?
      .to_vec();
    return Ok(
      out
        .into_iter()
        .map(|item| i16::from_sample(item))
        .collect::<Vec<i16>>(),
    );
  }
}
