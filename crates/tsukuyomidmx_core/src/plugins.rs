//! Plugin's trait definition, data structures etc.

use std::{
    fmt::Debug,
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::{Arc, RwLock},
};

use tokio::sync::watch;

use crate::prelude::{DmxAddress, UniverseId};

declare_id_newtype!(OutputPluginId);

/// Runtime of output plugin.
pub trait Plugin: Send + Sync + Debug {
    fn id(&self) -> OutputPluginId;

    fn start(self: Box<Self>) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>;
}

/// Message sent to running plugin.
#[derive(Debug, Clone)]
pub enum PluginMessage {
    DmxFrame(DmxFrame),
    Stop,
}

/// Static data about plugin configuration.
pub trait PluginInfo: Send + Sync + Debug {
    /// Create instance of plugin ([`Plugin`]).
    fn create_instance(&self, rx: watch::Receiver<PluginMessage>) -> Box<dyn Plugin>;

    fn universe(&self) -> UniverseId;
}

#[derive(derive_more::Debug)]
pub struct SpyPlugin {
    id: OutputPluginId,
    #[debug(skip)]
    data: Arc<RwLock<Vec<DmxFrame>>>,
    rx: watch::Receiver<PluginMessage>,
}

#[derive(Debug)]
pub struct SpyPluginInfo {
    universe: UniverseId,
    pub data: Arc<RwLock<Vec<DmxFrame>>>,
}

impl PluginInfo for SpyPluginInfo {
    fn create_instance(&self, rx: watch::Receiver<PluginMessage>) -> Box<dyn Plugin> {
        Box::new(SpyPlugin {
            id: OutputPluginId::new(),
            data: Arc::clone(&self.data),
            rx,
        })
    }

    fn universe(&self) -> UniverseId {
        self.universe
    }
}

impl SpyPluginInfo {
    pub fn new(universe: UniverseId) -> Self {
        Self {
            universe,
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

// TODO
impl Plugin for SpyPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn start(
        mut self: Box<SpyPlugin>,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async move {
            loop {
                self.rx.changed().await.unwrap();
                let frame = match self.rx.borrow_and_update().deref() {
                    PluginMessage::DmxFrame(frame) => frame.clone(),
                    PluginMessage::Stop => break,
                };
                self.data.write().unwrap().push(frame);
            }
            anyhow::Ok(())
        })
    }
}

#[derive(Debug, Clone)]
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

    /// Create [`DmxFrame`] filled with zero.
    pub fn zeros() -> Self {
        Self { data: [0; 512] }
    }
}

impl From<[u8; 512]> for DmxFrame {
    fn from(value: [u8; 512]) -> Self {
        Self { data: value }
    }
}
