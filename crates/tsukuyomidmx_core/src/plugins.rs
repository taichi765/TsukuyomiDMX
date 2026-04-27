//! Plugin's trait definition, data structures etc.

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::prelude::{DmxAddress, UniverseId};

declare_id_newtype!(OutputPluginId);

pub trait Plugin: Send + Sync + Debug {
    fn id(&self) -> OutputPluginId;

    fn start(&mut self) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

#[derive(derive_more::Debug)]
pub struct SpyPlugin {
    id: OutputPluginId,
    #[debug(skip)]
    pub data: Arc<RwLock<Vec<DmxFrame>>>,
}

// TODO
/*impl Plugin for SpyPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    async fn start(&mut self) -> Result<(), anyhow::Error> {
        loop {}
    }
}*/

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
    pub data: [u8; 512], // FIXME: &[u8]の方がいい？
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
