mod errors;
pub use errors::*;
mod commands;
pub use commands::*;
mod decider;
mod state;
pub use state::DocState;
mod def_registry;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
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
        Self {
            state: Arc::new(DocState::new()),
            subscribers: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            fixture_by_address_index: HashMap::new(),
        }
    }

    pub fn states(&self) -> Arc<DocState> {
        Arc::clone(&self.state)
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
        let cmd = decider::add_fixture(
            self.state.as_view(),
            fixture,
            &self.fixture_by_address_index,
        )?;
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
            self.state.as_view(),
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
        let cmd = decider::remove_fixture(self.state.as_view(), id)?;
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

    /// Callback called when [`DocEvent`] is occured.
    pub fn subscribe(&mut self, f: Box<dyn Fn(&DocEffect)>) {
        self.subscribers.push(f);
    }

    /// Helper method -- Applies [`DocEvent`] to `state` and `event_store`, then notifies event to subscribers.
    fn apply_command(&mut self, cmd: impl DocCommand) {
        let (undo, effect) = Box::new(cmd).apply(&self.state);
        self.undo_stack.push(undo);
        self.subscribers.iter().for_each(|f| f(&effect));
    }
}

/// UIとかに通知する用のやつ。
#[derive(Debug, Clone)]
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
