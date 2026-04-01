mod errors;
pub use errors::*;
mod commands;
pub use commands::*;
mod decider;
mod def_registry;
pub use def_registry::*;
mod state;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    doc::state::{AddressIndex, DocState},
    fixture::{Fixture, FixtureChange, FixtureId},
    fixture_def::FixtureDefId,
    functions::{FunctionData, FunctionId},
    prelude::ChannelDef,
    universe::{DmxAddress, UniverseId},
};
use tracing::{debug, instrument};

declare_id_newtype!(OutputPluginId);

/// Facade of [`DocState`].
///
/// Orchestrates `decider`, `commands`, `subscribers` etc.
/// This is in application layer.
#[derive(derive_more::Debug)]
pub struct Doc {
    #[debug(skip)]
    state: Arc<DocState>,
    #[debug(skip)]
    subscribers: Vec<Box<dyn Fn(&DocEffect)>>,
    undo_stack: Vec<Box<dyn DocCommand>>,
    redo_stack: Vec<Box<dyn DocCommand>>,
}

impl Doc {
    pub fn try_new() -> Result<Self, std::io::Error> {
        debug!("hello from core");
        let def_resource_path = {
            let mut p = dirs::data_local_dir().unwrap();
            p.push("tsukuyomidmx");
            p.push("fixtures");
            p
        };
        let state = DocState::new(Box::new(FixtureDefRegistryImpl::new(def_resource_path)));
        state.load_defs()?;

        Ok(Self {
            state: Arc::new(state),
            subscribers: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn new_with_def_registry(def_registry: Box<dyn FixtureDefRegistry>) -> Self {
        Self {
            state: Arc::new(DocState::new(def_registry)),
            subscribers: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn state_view(&self) -> DocStateView {
        DocStateView(Arc::clone(&self.state))
    }

    /// Callback called when [`DocEvent`] is occured.
    pub fn subscribe(&mut self, f: Box<dyn Fn(&DocEffect)>) {
        self.subscribers.push(f);
    }

    /// Undo.
    #[instrument]
    pub fn undo(&mut self) {
        let cmd = self
            .undo_stack
            .pop()
            .expect("undo stack is empty (todo: error返した方がいいかも)");
        let (redo, effect) = cmd.apply(&self.state);
        self.redo_stack.push(redo);
        self.subscribers.iter().for_each(|f| f(&effect));
    }

    /// Redo.
    #[instrument]
    pub fn redo(&mut self) {
        let cmd = self
            .redo_stack
            .pop()
            .expect("redo stack is empty (todo: error返した方がいいかも)");
        let (undo, effect) = cmd.apply(&self.state);
        self.undo_stack.push(undo);
        self.subscribers.iter().for_each(|f| f(&effect));
    }

    /// Adds fixture.
    #[instrument]
    pub fn add_fixture(&mut self, fixture: Fixture) -> Result<(), FixtureAddError> {
        let cmd = decider::add_fixture(self.state_view(), fixture)?;
        self.apply_command(cmd);
        Ok(())
    }

    /// Updates fixture.
    #[instrument]
    pub fn update_fixture(
        &mut self,
        id: FixtureId,
        change: FixtureChange,
    ) -> Result<(), FixtureUpdateError> {
        let cmd = decider::update_fixture(self.state_view(), id, change)?;
        self.apply_command(cmd);
        Ok(())
    }

    /// Removes fixture.
    /// If the fixture didn't exist, [FixtureRemoveError::FixtureNotFound][`FixtureRemoveError`] will be returned.
    #[instrument]
    pub fn remove_fixture(&mut self, id: &FixtureId) -> Result<(), FixtureRemoveError> {
        let cmd = decider::remove_fixture(self.state_view(), id)?;
        self.apply_command(cmd);
        Ok(())
    }

    #[instrument]
    pub fn add_function(&mut self, _value: FunctionData) -> Result<DocEffect, ()> {
        todo!()
    }

    #[instrument]
    pub fn update_function(&mut self, _new: FunctionData) -> Result<DocEffect, ()> {
        todo!()
    }

    #[instrument]
    pub fn remove_function(&mut self, _id: &FunctionId) -> Result<DocEffect, ()> {
        todo!()
    }

    /// 外部が関わる操作なのでDocCommandでのundo/redoはできない
    #[instrument]
    pub fn reload_defs(&mut self) -> Result<(), std::io::Error> {
        let ret = self.state.load_defs();
        if ret.is_ok() {
            self.subscribers
                .iter()
                .for_each(|f| f(&DocEffect::DefRegistryLoaded));
        }
        ret
    }

    /// Helper method -- Applies [`DocEvent`] to `state` and `event_store`, then notifies event to subscribers.
    fn apply_command(&mut self, cmd: impl DocCommand) {
        let (undo, effect) = Box::new(cmd).apply(&self.state);
        self.undo_stack.push(undo);
        self.subscribers.iter().for_each(|f| f(&effect));
    }
}

/// Readonly-facade of [`DocState`].
pub struct DocStateView(Arc<DocState>);

impl Clone for DocStateView {
    /// Cheap clone.
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

// RwLock helpers
impl DocStateView {
    pub fn with_fixtures<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>) -> R,
    {
        self.0.with_fixtures(f)
    }

    pub fn with_fixture_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn FixtureDefRegistry) -> R,
    {
        self.0.with_fixture_defs(f)
    }

    pub fn with_functions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FunctionId, FunctionData>) -> R,
    {
        self.0.with_functions(f)
    }

    pub fn with_fixtures_and_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>, &dyn FixtureDefRegistry) -> R,
    {
        self.0.with_fixtures_and_defs(f)
    }

    pub fn with_address_index<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AddressIndex) -> R,
    {
        self.0.with_address_index(f)
    }

    pub fn with_universe_settings<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<UniverseId, UniverseSetting>) -> R,
    {
        self.0.with_universe_settings(f)
    }
}

// Utilities
impl DocStateView {
    ///
    pub fn resolve_address(
        &self,
        fixture_id: FixtureId,
        channel: &str,
    ) -> Result<ResolvedAddress, ResolveError> {
        let (channel_def, address) =
            self.with_fixtures_and_defs(|fxts, defs| -> Result<_, ResolveError> {
                let fxt = fxts.get(&fixture_id).ok_or(ResolveError::FixtureNotFound(
                    FixtureNotFoundError(fixture_id),
                ))?;
                let def = defs
                    .get(&fxt.fixture_def())
                    .expect("invariant: definition must exist");
                let mode = def
                    .mode(fxt.fixture_mode())
                    .expect("invariant: mode must exist");
                let offset =
                    mode.get_offset_by_channel(channel)
                        .ok_or(ResolveError::ChannelNotFound {
                            fixture_def: fxt.fixture_def().to_owned(),
                            mode: fxt.fixture_mode().to_string(),
                            channel: channel.to_string(),
                        })?;
                let channel_def = def
                    .channel_template(channel)
                    .expect("channel order and channel template must match")
                    .to_owned();
                let address = mode
                    .occupied_addresses(fxt.universe_id(), fxt.address())
                    .nth(offset)
                    .unwrap(); // TODO: この辺の処理はFixtureModeのメソッドにしたい
                Ok((channel_def, address))
            })?;

        Ok(ResolvedAddress {
            channel_def,
            universe: address.0,
            address: address.1,
        })
    }

    /// Returns max address which is occupied by a fixture.
    ///
    /// If there's no fixture in the universe, `None` is returned.
    /// If universe does not exist in the DocStore, `None` is returned.
    pub fn current_max_address(&self, universe: UniverseId) -> Option<DmxAddress> {
        self.with_address_index(|index| {
            index
                .iter()
                .filter(|((u_id, _), _)| *u_id == universe)
                .map(|e| e.0.1)
                .max()
        })
    }
}

/// UIとかに通知する用のやつ。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DocEffect {
    UniverseSettingsChanged,
    UniverseAdded(UniverseId),
    UniverseRemoved(UniverseId),

    FixtureAdded(FixtureId),
    FixtureUpdated(FixtureId),
    FixtureRemoved(FixtureId),

    FixtureDefAdded(FixtureDefId),
    FixtureDefUpdated(FixtureDefId),
    FixtureDefRemoved(FixtureDefId),

    FunctionAdded(FunctionId),
    FunctionUpdated(FunctionId),
    FunctionRemoved(FunctionId),

    AddressIndexChanged((UniverseId, DmxAddress), (FixtureId, usize)),
    DefRegistryLoaded,
}

#[derive(Debug)]
pub struct ResolvedAddress {
    pub channel_def: ChannelDef,
    pub universe: UniverseId,
    pub address: DmxAddress,
}

pub trait EventStore {
    fn append(&self, event: DocEffect);
}

pub struct UniverseSetting {
    output_plugins: HashSet<OutputPluginId>,
}

impl UniverseSetting {
    pub fn new() -> Self {
        Self {
            output_plugins: HashSet::new(),
        }
    }

    pub fn output_plugins(&self) -> &HashSet<OutputPluginId> {
        &self.output_plugins
    }
}

#[cfg(test)]
mod tests {
    mod commands;
    mod decider;
    mod events;
    //mod fixtures;
    //mod functions;
    mod helpers;
    //mod resolve;
    //mod universe_outputs;
}
