use std::{
    io::ErrorKind,
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
};

use anyhow::Context;
use ola::{
    DmxBuffer,
    config::{Config, ConnectError},
};
use tokio::sync::watch;
use tsukuyomidmx_core::plugins::BlockingPlugin;

use tsukuyomidmx_core::{
    plugins::{DmxFrame, OutputPluginId, Plugin},
    prelude::UniverseId,
};

#[derive(Debug)]
pub struct OlaPlugin {
    id: OutputPluginId,
    universe: UniverseId,
    rx: watch::Receiver<[u8; 512]>,
}

impl Plugin for OlaPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    async fn start(&mut self) -> Result<(), anyhow::Error> {
        let config = Config::new();
        let mut client = config
            .connect()
            .with_context(|| format!("failed to connect to olad at {}", "9010"))?;
        let mut buf = DmxBuffer::from([0; 512]);
        loop {
            self.rx.changed().await.unwrap();
            let frame = self.rx.borrow_and_update();
            *buf = frame.clone();

            client
                .send_dmx(self.universe.as_usize() as u32, &buf)
                .with_context(|| format!("failed to send dmx data to olad"))?;
        }
    }
}

impl OlaPlugin {
    pub fn new(universe: UniverseId, rx: watch::Receiver<[u8; 512]>) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            rx,
        }
    }
}
