use std::{ops::Deref, pin::Pin};

use anyhow::Context;
use ola::{DmxBuffer, config::Config};
use tokio::sync::watch;
use tsukuyomidmx_core::plugins::PluginMessage;

use tsukuyomidmx_core::{
    plugins::{OutputPluginId, Plugin},
    prelude::UniverseId,
};

#[derive(Debug)]
pub struct OlaPlugin {
    id: OutputPluginId,
    universe: UniverseId,
    rx: watch::Receiver<PluginMessage>,
}

impl Plugin for OlaPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn start(
        mut self: Box<Self>,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async move {
            let config = Config::new();
            let mut client = config
                .connect()
                .with_context(|| format!("failed to connect to olad at {}", "9010"))?;
            let mut buf = DmxBuffer::from([0; 512]);
            loop {
                self.rx.changed().await.unwrap();
                let frame = match self.rx.borrow_and_update().deref() {
                    PluginMessage::DmxFrame(frame) => frame.as_slice().to_owned(),
                    PluginMessage::Stop => break,
                };
                *buf = frame;

                client
                    .send_dmx(self.universe.as_usize() as u32, &buf)
                    .with_context(|| format!("failed to send dmx data to olad"))?;
            }
            anyhow::Ok(())
        })
    }
}

impl OlaPlugin {
    pub fn new(universe: UniverseId, rx: watch::Receiver<PluginMessage>) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            rx,
        }
    }
}
