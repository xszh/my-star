use std::{sync::Arc, time::Duration};

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
  // buffer: Arc<RwLock<Vec<f32>>>,
  sx: Arc<Mutex<Sender<Vec<f32>>>>,
  rx: Receiver<Vec<f32>>,
}

impl ShortRecord {
  pub fn new() -> Result<Self> {
    let host = cpal::default_host();
    let device = host
      .default_input_device()
      .ok_or(anyhow!("no default input device"))?;

    let config = device.default_input_config()?;

    let (sx, rx) = unbounded::<Vec<f32>>();

    Ok(Self {
      device,
      config,
      // buffer: Arc::default(),
      sx: Arc::new(Mutex::new(sx)),
      rx,
    })
  }

  fn run<F>(&self, buffer: Arc<Mutex<Vec<f32>>>) -> Result<cpal::Stream>
  where
    F: SizedSample,
    f32: FromSample<F>,
  {
    let channel = self.config.clone().channels();
    Ok(self.device.build_input_stream(
      &self.config.clone().into(),
      move |data: &[F], _: &_| {
        Self::write_data(data, &mut buffer.lock(), channel);
      },
      |e| println!("input stream fail: {}", e),
      None,
    )?)
  }

  pub fn start(&self) -> Result<()> {
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::default();

    let stream = match self.config.sample_format() {
      SampleFormat::I16 => self.run::<i16>(buffer.clone()),
      SampleFormat::I32 => self.run::<i32>(buffer.clone()),
      SampleFormat::I64 => self.run::<i64>(buffer.clone()),
      SampleFormat::U8 => self.run::<u8>(buffer.clone()),
      SampleFormat::U16 => self.run::<u16>(buffer.clone()),
      SampleFormat::U32 => self.run::<u32>(buffer.clone()),
      SampleFormat::U64 => self.run::<u64>(buffer.clone()),
      SampleFormat::F32 => self.run::<f32>(buffer.clone()),
      SampleFormat::F64 => self.run::<f64>(buffer.clone()),
      s => Err(anyhow!("unsupported sample format: {}", s)),
    }?;

    stream.play()?;

    loop {
      if let Err(e) = self.rx.try_recv() {
        if e.is_empty() {
          continue;
        }
      }
      break;
    }
    drop(stream);

    let fin_buffer = buffer.clone();
    self.sx.clone().lock().send(fin_buffer.lock().to_vec())?;

    Ok(())
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
    self.sx.clone().lock().send(vec![])?;

    let raw = self.rx.recv_timeout(Duration::from_secs(5))?;

    let mut resampler =
      resample::resample::<f32>(self.config.sample_rate().0 as f64, 16000f64, raw.len())?;
    let mut out = resampler.output_buffer_allocate(true);
    resampler.process_into_buffer(&vec![raw], &mut out, None)?;

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
}
