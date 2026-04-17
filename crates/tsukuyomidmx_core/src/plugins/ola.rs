use std::{
    io::ErrorKind,
    sync::mpsc::{self, Sender},
    thread::JoinHandle,
};

use ola::{DmxBuffer, config::ConnectError};

use crate::{
    plugins::Plugin,
    prelude::{OutputPluginId, UniverseId},
};

use super::DmxFrame;

#[derive(Debug)]
pub struct OlaPlugin {
    id: OutputPluginId,
    tx: Sender<(u32, [u8; 512])>,
    join_handle: JoinHandle<Result<(), std::io::Error>>,
}

impl Plugin for OlaPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn send_dmx(&self, universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error> {
        self.tx
            .send((universe_id.value().into(), dmx_data.as_slice().clone()))
            .unwrap();
        // TODO: map error
        Ok(())
    }
}

impl OlaPlugin {
    pub fn new() -> Result<Self, ConnectError> {
        let (tx, rx) = mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let cfg = ola::config::Config::new();
            let mut client = cfg
                .connect()
                .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;

            while let Ok((univ, data)) = rx.recv() {
                let buf = DmxBuffer::from(data);
                client
                    .send_dmx(univ, &buf)
                    .map_err(|e| std::io::Error::new(ErrorKind::TimedOut, e))?;
            }
            Ok(())
        });

        Ok(Self {
            id: OutputPluginId::new(),
            tx,
            join_handle,
        })
    }
}
