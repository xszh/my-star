use std::sync::Arc;

use crate::resample;
use anyhow::{anyhow, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, Sample, SampleFormat, SizedSample,
};
use parking_lot::RwLock;
use rubato::Resampler;

pub struct ShortRecord {
  device: cpal::Device,
  config: cpal::SupportedStreamConfig,
  stream: Option<cpal::Stream>,
  buffer: Arc<RwLock<Vec<f32>>>,
}

impl ShortRecord {
  pub fn new() -> Result<Self> {
    let host = cpal::default_host();
    let device = host
      .default_input_device()
      .ok_or(anyhow!("no default input device"))?;

    let config = device.default_input_config()?;

    Ok(Self {
      device,
      config,
      stream: None,
      buffer: Arc::default(),
    })
  }

  fn run<F>(&mut self) -> Result<cpal::Stream>
  where
    F: SizedSample,
    f32: FromSample<F>,
  {
    let w_buffer = self.buffer.clone();
    let channel = self.config.clone().channels();
    Ok(self.device.build_input_stream(
      &self.config.clone().into(),
      move |data: &[F], _: &_| {
        Self::write_data(data, &mut w_buffer.write(), channel);
      },
      |e| println!("input stream fail: {}", e),
      None,
    )?)
  }

  pub fn start(&mut self) -> Result<()> {
    self.drop()?;

    self.stream = Some(match self.config.sample_format() {
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
    }?);

    let stream = self.stream.as_ref().unwrap();
    stream.play()?;

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

  fn drop(&mut self) -> Result<Vec<f32>> {
    self.stream.take().map(|s| drop(s));
    let raw = self.buffer.read().to_vec();
    self.buffer = Arc::default();

    Ok(raw)
  }

  pub fn stop(&mut self) -> Result<Vec<i16>> {
    let raw = self.drop()?;

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
