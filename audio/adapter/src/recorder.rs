use std::sync::Arc;

use anyhow::{anyhow, Result};
use cpal::{
  traits::{DeviceTrait, HostTrait},
  FromSample, Sample, SizedSample,
};
use parking_lot::RwLock;

pub struct Recorder {
  device: cpal::Device,
  config: cpal::SupportedStreamConfig,
  stream: Option<cpal::Stream>,
  buffer: Arc<RwLock<Vec<f32>>>,
}

impl Recorder {
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
  pub fn start(&mut self) -> Result<()> {
    self.stop()?;

    let w_buffer = self.buffer.clone();
    let channel = self.config.clone().channels();

    self.device.build_input_stream(
      &self.config.clone().into(),
      move |data: &[f32], _: &_| {
        Self::write_data(data, &mut w_buffer.write(), channel);
      },
      |e| println!("input stream fail: {}", e),
      None,
    )?;

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
      d += data[idx].to_sample::<f32>() / (channel as f32);
      c_idx += 1;
      if c_idx == channel {
        buffer.push(d);
        c_idx = 1;
      }
      idx += 1;
    }
  }
  pub fn stop(&mut self) -> Result<()> {
    Ok(())
  }
}
