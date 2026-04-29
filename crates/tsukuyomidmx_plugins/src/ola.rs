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
    port: u16,
}

impl Plugin for OlaPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn universe(&self) -> UniverseId {
        self.universe
    }

    fn start(
        self: Box<Self>,
        mut rx: watch::Receiver<PluginMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async move {
            let mut config = Config::new();
            config.server_port = self.port;
            let mut client = config
                .connect_async()
                .await
                .with_context(|| format!("failed to connect to olad at {}", "9010"))?;
            let mut buf = DmxBuffer::from([0; 512]);
            loop {
                rx.changed().await.unwrap();
                let frame = match rx.borrow_and_update().deref() {
                    PluginMessage::DmxFrame(frame) => frame.as_slice().to_owned(),
                    PluginMessage::Stop => break,
                };
                *buf = frame;

                client
                    .send_dmx_streaming(self.universe.as_usize() as u32, &buf)
                    .await
                    .with_context(|| format!("failed to send dmx data to olad"))?;
            }
            anyhow::Ok(())
        })
    }
}

impl OlaPlugin {
    pub fn new(universe: UniverseId) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            port: 9010,
        }
    }

    pub fn new_with_port(universe: UniverseId, port: u16) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            port,
        }
    }
}
