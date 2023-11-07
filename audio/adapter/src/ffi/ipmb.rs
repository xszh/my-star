use std::sync::Arc;

use anyhow::Result;
use ipmb::{label, BytesMessage, EndpointReceiver, EndpointSender, Options, Message};
use parking_lot::Mutex;

use crate::Adapter;

pub struct EndPoint {
  pub sx: Arc<Mutex<EndpointSender<BytesMessage>>>,
  pub rx: EndpointReceiver<BytesMessage>,
}

impl EndPoint {
  pub fn new() -> Result<Self> {
    let (sx, rx) = ipmb::join::<BytesMessage, BytesMessage>(
      Options::new("mystar.audio.adapter", label!("control"), ""),
      None,
    )?;
    Ok(Self {
      sx: Arc::new(Mutex::new(sx)),
      rx: rx,
    })
  }

  pub fn launch(&mut self) -> Result<()> {
    while let Ok(message) = self.rx.recv(None) {
      let payload: BytesMessage = message.payload;
      // match payload.format {
        
      // }
    };
    Ok(())
  }
}

impl Adapter for EndPoint {
  fn start_record() -> Result<()> {
    todo!()
  }

  fn stop_record() -> Result<()> {
    todo!()
  }
}
