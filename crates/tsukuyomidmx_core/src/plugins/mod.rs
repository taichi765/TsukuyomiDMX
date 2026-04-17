use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::{
    doc::OutputPluginId,
    universe::{DmxAddress, UniverseId},
};

pub mod artnet;
mod ola;
pub use ola::OlaPlugin;

pub trait Plugin: Send + Sync + Debug {
    fn id(&self) -> OutputPluginId;

    fn send_dmx(&self, universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error>;
}

#[derive(derive_more::Debug)]
pub struct SpyPlugin {
    id: OutputPluginId,
    #[debug(skip)]
    pub data: Arc<RwLock<Vec<DmxFrame>>>,
}

impl Plugin for SpyPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn send_dmx(&self, _universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error> {
        self.data.write().unwrap().push(dmx_data);
        Ok(())
    }
}

impl SpyPlugin {
    pub fn new() -> Self {
        Self {
            id: OutputPluginId::new(),
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug)]
pub struct DmxFrame {
    data: [u8; 512], // FIXME: &[u8]の方がいい？
}

impl DmxFrame {
    pub fn iter(&self) -> impl Iterator<Item = (DmxAddress, u8)> {
        // index -> address conversion
        self.data
            .iter()
            .enumerate()
            .map(|(idx, v)| (DmxAddress::new(idx + 1).unwrap(), *v))
    }

    /// If you are dealing with [`DmxAddress`], it's recommended to use [`DmxFrame::iter()`] instead.
    pub fn as_slice(&self) -> &[u8; 512] {
        &self.data
    }
}

impl From<[u8; 512]> for DmxFrame {
    fn from(value: [u8; 512]) -> Self {
        Self { data: value }
    }
}
