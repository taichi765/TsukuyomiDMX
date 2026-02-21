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
    path::PathBuf,
    sync::Arc,
};

use crate::{
    doc::state::DocState,
    fixture::{Fixture, FixtureChange, FixtureId, MergeMode},
    fixture_def::FixtureDefId,
    functions::{FunctionData, FunctionId},
    universe::{DmxAddress, UniverseId},
};

declare_id_newtype!(OutputPluginId);

/// Facade of [`DocState`].
///
/// Orchestrates `decider`, `commands`, `subscribers` etc.
/// This is in application layer.
pub struct Doc {
    state: Arc<DocState>,
    subscribers: Vec<Box<dyn Fn(&DocEffect)>>,
    undo_stack: Vec<Box<dyn DocCommand>>,
    redo_stack: Vec<Box<dyn DocCommand>>,

    fixture_by_address_index: HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>, // TODO: prokectionに移す
}

impl Doc {
    pub fn new() -> Self {
        let path: PathBuf = [
            r"C:\",
            "Users",
            "taich",
            "source",
            "tsukuyomi-rs",
            "resources",
            "fixture_defs",
        ]
        .iter()
        .collect(); // TODO: dirsクレートを使う
        let state = DocState::new(Box::new(FixtureDefRegistryImpl::new(path)));
        state
            .load_defs()
            .expect("TODO: 初回のloadはUI側に呼ばせてもいいかも");
        Self {
            state: Arc::new(state),
            subscribers: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            fixture_by_address_index: HashMap::new(),
        }
    }

    pub fn new_with_def_registry(def_registry: Box<dyn FixtureDefRegistry>) -> Self {
        Self {
            state: Arc::new(DocState::new(def_registry)),
            subscribers: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            fixture_by_address_index: HashMap::new(),
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
    pub fn add_fixture(&mut self, fixture: Fixture) -> Result<(), FixtureAddError> {
        let cmd = decider::add_fixture(self.state_view(), fixture, &self.fixture_by_address_index)?;
        self.apply_command(cmd);
        Ok(())
    }

    /// Updates fixture.
    pub fn update_fixture(
        &mut self,
        id: FixtureId,
        change: FixtureChange,
    ) -> Result<(), FixtureUpdateError> {
        let cmd = decider::update_fixture(
            self.state_view(),
            id,
            change,
            &self.fixture_by_address_index,
        )?;
        self.apply_command(cmd);
        Ok(())
    }

    /// Removes fixture.
    /// If the fixture didn't exist, [FixtureRemoveError::FixtureNotFound][`FixtureRemoveError`] will be returned.
    pub fn remove_fixture(&mut self, id: &FixtureId) -> Result<(), FixtureRemoveError> {
        let cmd = decider::remove_fixture(self.state_view(), id)?;
        self.apply_command(cmd);
        Ok(())
    }

    pub fn add_function(&mut self, _value: FunctionData) -> Result<DocEffect, ()> {
        todo!()
    }

    pub fn update_function(&mut self, _new: FunctionData) -> Result<DocEffect, ()> {
        todo!()
    }

    pub fn remove_function(&mut self, _id: &FunctionId) -> Result<DocEffect, ()> {
        todo!()
    }

    /// 外部が関わる操作なのでDocCommandでのundo/redoはできない
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

#[derive(Debug)]
pub struct ResolvedAddress {
    pub merge_mode: MergeMode,
    pub address: DmxAddress,
}

#[cfg(test)]
mod tests {
    mod address_index;
    mod current_max_address;
    mod events;
    mod fixtures;
    mod functions;
    mod helpers;
    mod resolve;
    mod universe_outputs;
}
