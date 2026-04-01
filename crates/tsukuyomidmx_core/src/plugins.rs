use std::fmt::Debug;

use crate::{
    doc::OutputPluginId,
    universe::{DmxAddress, UniverseId},
};

pub mod artnet;

pub trait Plugin: Send + Sync {
    fn send_dmx(&self, universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error>;
    fn id(&self) -> OutputPluginId;
}

impl Debug for dyn Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

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
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl From<[u8; 512]> for DmxFrame {
    fn from(value: [u8; 512]) -> Self {
        Self { data: value }
    }
}
