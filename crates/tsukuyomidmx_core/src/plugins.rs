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

/// Output plugin.
pub trait Plugin: Send + Sync + Debug {
    fn id(&self) -> OutputPluginId;

    fn universe(&self) -> UniverseId;

    /// This function is called in [task::spawn()][tokio::task::spawn()], so
    /// you should not perform a blocking operation.
    /// If you need it, use [task::spawn_blocking()][tokio::task::spawn_blocking()]
    /// inside the function.
    fn start(
        self: Box<Self>,
        rx: watch::Receiver<PluginMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>>;
}

/// Message sent to running plugin.
#[derive(Debug, Clone)]
pub enum PluginMessage {
    DmxFrame(DmxFrame),
    Stop,
}

#[derive(derive_more::Debug, Clone)]
pub struct SpyPlugin {
    id: OutputPluginId,
    universe: UniverseId,
    #[debug(skip)]
    pub data: Arc<RwLock<Vec<DmxFrame>>>,
}

impl Plugin for SpyPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn universe(&self) -> UniverseId {
        self.universe
    }

    fn start(
        self: Box<SpyPlugin>,
        mut rx: watch::Receiver<PluginMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async move {
            loop {
                rx.changed().await.unwrap();
                let frame = match rx.borrow_and_update().deref() {
                    PluginMessage::DmxFrame(frame) => frame.clone(),
                    PluginMessage::Stop => break,
                };
                self.data.write().unwrap().push(frame);
            }
            anyhow::Ok(())
        })
    }
}

impl SpyPlugin {
    pub fn new(universe: UniverseId) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            data: Arc::new(RwLock::new(Vec::new())),
        }
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
