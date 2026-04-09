//! Validates command and returns [`DocEvent`] or an error.
//! Similar to event sourcing's decider.

use std::fmt::Debug;

use super::DocStateView;
use super::errors::*;
use crate::doc::OutputPluginInfo;
use crate::doc::commands::*;
use crate::doc::state::AddressIndex;
use crate::fixture::FixtureChange;
use crate::fixture_def::AddressIter;
use crate::functions::{AppliedFunctionId, Function};
use crate::prelude::*;

pub(super) fn add_fixture(
    state: DocStateView,
    fixture: Fixture,
) -> Result<
    AddFixtureCommand<impl Iterator<Item = (UniverseId, DmxAddress)> + Clone + Debug>,
    FixtureAddError,
> {
    if state.with_fixtures(|it| it.contains_key(&fixture.id())) {
        return Err(FixtureAddError::FixtureAlreadyExists(fixture.id()));
    }

    let def_id = fixture.fixture_def();
    let occupied_addresses = state.with_fixture_defs(|defs| {
        let def = defs.get(&def_id).map_err(|e| {
            FixtureAddError::FixtureDefNotFound(FixtureDefNotFoundError {
                fixture_id: fixture.id(),
                fixture_def_id: def_id.clone(),
                source: e,
            })
        })?;
        let mode = def
            .mode(fixture.fixture_mode())
            .ok_or(FixtureAddError::ModeNotFound(ModeNotFoundError {
                fixture_def: def_id.clone(),
                mode: fixture.fixture_mode().to_string(),
            }))?;

        Ok::<_, FixtureAddError>(mode.occupied_addresses(fixture.universe_id(), fixture.address()))
    })?;

    state.with_address_index(|index| {
        validate_fixture_address(fixture.id(), occupied_addresses.clone(), index)
            .map_err(|e| FixtureAddError::AddressValidateError(e))
    })?;

    Ok(AddFixtureCommand::new(fixture, occupied_addresses))
}

pub(super) fn update_fixture(
    state: DocStateView,
    id: FixtureId,
    change: FixtureChange,
) -> Result<
    UpdateFixtureCommand<impl Iterator<Item = (UniverseId, DmxAddress)> + Clone + Debug>,
    FixtureUpdateError,
> {
    let new_occupied_addresses = state.with_fixtures_and_defs(|fxts, defs| {
        let fxt = fxts.get(&id).ok_or(FixtureNotFoundError(id))?;
        let def = defs.get(&fxt.fixture_def()).unwrap();
        let new_occupied_addresses = compute_occupied_addresses(fxt, def, &change)?;

        // Validate new addresses
        match change {
            FixtureChange::Address(_) | FixtureChange::Mode(_) | FixtureChange::Universe(_) => {
                state
                    .with_address_index(|index| {
                        validate_fixture_address(fxt.id(), new_occupied_addresses.clone(), index)
                            .map_err(|e| FixtureUpdateError::AddressValidateError(e))
                    })
                    .map(|_| new_occupied_addresses)
            }
            _ => Ok(new_occupied_addresses),
        }
    })?;

    let old_occupied_addresses = state.with_fixtures_and_defs(|fxts, defs| {
        let fxt = fxts.get(&id).unwrap();
        defs.get(&fxt.fixture_def())
            .unwrap()
            .mode(fxt.fixture_mode())
            .unwrap()
            .occupied_addresses(fxt.universe_id(), fxt.address())
    });

    Ok(UpdateFixtureCommand::new(
        id,
        change,
        old_occupied_addresses,
        new_occupied_addresses,
    ))
}

pub(super) fn remove_fixture(
    state: DocStateView,
    id: &FixtureId,
) -> Result<RemoveFixtureCommand, FixtureRemoveError> {
    if !state.with_fixtures(|it| it.contains_key(id)) {
        return Err(FixtureRemoveError::FixtureNotFound(FixtureNotFoundError(
            *id,
        )));
    }

    Ok(RemoveFixtureCommand::new(*id))
}

pub(super) fn add_function(
    state: DocStateView,
    fun: Function,
) -> Result<AddFunctionCommand, AddFunctionError> {
    if state.with_functions(|it| it.contains_key(&fun.id())) {
        return Err(AddFunctionError::IdAlreadyUsed(fun.id()));
    }

    Ok(AddFunctionCommand::new(fun))
}

pub(super) fn update_function(_state: DocStateView, _new: Function) -> Result<(), ()> {
    todo!()
}

pub(super) fn remove_function(
    state: DocStateView,
    id: AppliedFunctionId,
) -> Result<RemoveFunctionCommand, RemoveFunctionError> {
    if !state.with_functions(|it| it.contains_key(&id)) {
        return Err(RemoveFunctionError::FunctionNotFound(id));
    }

    Ok(RemoveFunctionCommand::new(id))
}

pub(super) fn add_output_plugin(
    state: DocStateView,
    universe: UniverseId,
    plugin: OutputPluginInfo,
) -> Result<AddOutputPluginCommand, AddOutputPluginError> {
    if !state.with_universe_settings(|it| it.contains_key(&universe)) {
        return Err(AddOutputPluginError::UniverseNotFound(universe));
    }

    Ok(AddOutputPluginCommand::new(universe, plugin))
}

pub(super) fn remove_output_plugin(
    _state: DocStateView,
) -> Result<RemoveOutputPluginCommand, RemoveOutputPluginError> {
    todo!()
}

/// changeを適用したあとのoccupied_addressを計算する
fn compute_occupied_addresses(
    fixture: &Fixture,
    def: &FixtureDef,
    change: &FixtureChange,
) -> Result<AddressIter, ModeNotFoundError> {
    match change {
        FixtureChange::Mode(mode_name) => {
            let mode = def.mode(mode_name).ok_or(ModeNotFoundError {
                fixture_def: def.id().clone(),
                mode: mode_name.clone(),
            })?;
            Ok(mode.occupied_addresses(fixture.universe_id(), fixture.address()))
        }
        FixtureChange::Address(adr) => {
            let mode = def
                .mode(fixture.fixture_mode())
                .expect("invariant: mode must exist");
            Ok(mode.occupied_addresses(fixture.universe_id(), *adr))
        }
        FixtureChange::Universe(u_id) => {
            let mode = def
                .mode(fixture.fixture_mode())
                .expect("invariant: mode must exist");
            Ok(mode.occupied_addresses(*u_id, fixture.address()))
        }
        _ => {
            let mode = def
                .mode(fixture.fixture_mode())
                .expect("invariant: mode must exist");
            Ok(mode.occupied_addresses(fixture.universe_id(), fixture.address()))
        }
    }
}

/// Validates that the fixture does not conflict with existing [Fixture]s' address.
fn validate_fixture_address(
    fixture_id: FixtureId,
    occupied_addresses: impl Iterator<Item = (UniverseId, DmxAddress)>,
    address_index: &AddressIndex,
) -> Result<(), ValidateError> {
    let mut conflicts = Vec::new();

    for (new_offset, (new_uni, new_adr)) in occupied_addresses.enumerate() {
        if let Some((existing_fixture_id, offset)) = address_index.get(&(new_uni, new_adr)) {
            if *existing_fixture_id == fixture_id {
                continue;
            }
            conflicts.push(AddressConflictedError {
                universe: new_uni,
                address: new_adr,
                existing_fixture_id: *existing_fixture_id,
                existing_offset: *offset,
                new_fixture_id: fixture_id,
                new_offset,
            });
        }
    }

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(ValidateError::AddressConflicted(conflicts))
    }
}
