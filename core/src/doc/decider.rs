//! Validates command and returns [`DocEvent`] or an error.
//! Similar to event sourcing's decider.

use std::collections::HashMap;

use super::errors::*;
use super::{DocEffect, state::DocStateView};
use crate::doc::commands::{AddFixtureCommand, RemoveFixtureCommand, UpdateFixtureCommand};
use crate::fixture::FixtureChange;
use crate::functions::FunctionData;
use crate::prelude::*;

pub(super) fn add_fixture(
    state: DocStateView,
    fixture: Fixture,
    fixture_by_address_index: &HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
) -> Result<AddFixtureCommand, FixtureAddError> {
    if state.with_fixtures(|it| it.contains_key(&fixture.id())) {
        return Err(FixtureAddError::FixtureAlreadyExists(fixture.id()));
    }

    let def_id = fixture.fixture_def();
    let occupied_addresses = state.with_fixture_defs(|defs| {
        let def = defs
            .get(&def_id)
            .ok_or(FixtureAddError::FixtureDefNotFound(FixtureDefNotFound {
                fixture_id: fixture.id(),
                fixture_def_id: def_id,
            }))?;
        let mode = def
            .mode(fixture.fixture_mode())
            .ok_or(FixtureAddError::ModeNotFound(ModeNotFound {
                fixture_def: def_id,
                mode: fixture.fixture_mode().to_string(),
            }))?;
        Ok::<_, FixtureAddError>(
            mode.occupied_addresses(fixture.address())
                .collect::<Vec<_>>(),
        )
    })?;

    validate_fixture_address(&fixture, &occupied_addresses, fixture_by_address_index)
        .map_err(|e| FixtureAddError::AddressValidateError(e))?;

    Ok(AddFixtureCommand::new(fixture))
}

pub(super) fn update_fixture(
    state: DocStateView,
    id: FixtureId,
    change: FixtureChange,
    fixture_by_address_index: &HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
) -> Result<UpdateFixtureCommand, FixtureUpdateError> {
    state.with_fixtures_and_defs(|fxts, defs| -> Result<(), FixtureUpdateError> {
        let fxt = fxts.get(&id).ok_or(FixtureNotFound(id))?;
        let def = defs.get(&fxt.fixture_def()).unwrap();
        let occupied_addresses = compute_occupied_addresses(fxt, def, &change)?;

        validate_fixture_address_change(
            fxt,
            &change,
            &occupied_addresses,
            &fixture_by_address_index,
        )
        .map_err(|e| FixtureUpdateError::AddressValidateError(e))
    })?;

    // TODO: projectionに移す
    /*for adr in occupied_addresses {
        if let Some(_) = self.fixture_by_address_index.insert(
            (fixture.universe_id(), adr),
            (fixture.id(), adr.checked_sub(fixture.address()).unwrap()),
        ) {
            warn!("there must be logic error in address validation");
        }
    }*/
    Ok(UpdateFixtureCommand::new(id, change))
}

pub(super) fn remove_fixture(
    state: DocStateView,
    id: &FixtureId,
) -> Result<RemoveFixtureCommand, FixtureRemoveError> {
    if !state.with_fixtures(|it| it.contains_key(id)) {
        return Err(FixtureRemoveError::FixtureNotFound(FixtureNotFound(*id)));
    }

    // TODO: Projectionに移す
    /*let fixture = state
        .fixtures
        .get(id)
        .expect("we checked with contains_key()");
    let def_id = fixture.fixture_def();
    let fixture_def = state
        .fixture_defs
        .get(&def_id)
        .expect("invariant: FixtureDef must exist");
    let occupied_addresses = fixture
    .occupied_addresses(fixture_def)
    .expect("invariant: mode must exist");
    for adr in occupied_addresses {
        if let Some((old_id, offset)) = self
            .fixture_by_address_index
            .remove(&(fixture.universe_id(), adr))
        {
            // FIXME: unwrap
            if old_id != *id || offset != adr.checked_sub(fixture.address()).unwrap() {
                warn!(address=?adr,fixture_id=?id,?old_id,?offset,"address index had unexpected value");
            }
        } else {
            warn!("the states of address index was invalid");
        }
    }*/

    Ok(RemoveFixtureCommand::new(*id))
}

pub(super) fn add_fixture_def(_state: DocStateView, _def: FixtureDef) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn update_fixture_def(_state: DocStateView, _new: FixtureDef) -> Result<DocEffect, ()> {
    todo!()
}

// TODO: このFixtureDefを使っているFixtureをどうするか
pub(super) fn remove_fixture_def(
    _state: DocStateView,
    _id: &FixtureDefId,
) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn add_function(_state: DocStateView, _value: FunctionData) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn update_function(_state: DocStateView, _new: FunctionData) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn remove_function(_state: DocStateView, _id: &FunctionId) -> Result<DocEffect, ()> {
    todo!()
}

fn compute_occupied_addresses(
    fixture: &Fixture,
    def: &FixtureDef,
    change: &FixtureChange,
) -> Result<Vec<DmxAddress>, ModeNotFound> {
    match change {
        FixtureChange::Mode(mode_name) => {
            let mode = def.mode(mode_name).ok_or(ModeNotFound {
                fixture_def: def.id(),
                mode: mode_name.clone(),
            })?;
            Ok(mode.occupied_addresses(fixture.address()).collect())
        }
        FixtureChange::Address(adr) => {
            let mode = def
                .mode(fixture.fixture_mode())
                .expect("invariant: mode must exist");
            Ok(mode.occupied_addresses(*adr).collect())
        }
        _ => {
            let mode = def
                .mode(fixture.fixture_mode())
                .expect("invariant: mode must exist");
            Ok(mode.occupied_addresses(fixture.address()).collect())
        }
    }
}

/// Validates that the fixture does not conflict with existing [Fixture]s' address.
/// This is a helper function to call [`validate_fixture_address_with_params()`].
fn validate_fixture_address(
    fixture: &Fixture,
    occupied_addresses: &[DmxAddress],
    fixture_by_address_index: &HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
) -> Result<(), ValidateError> {
    validate_fixture_address_with_params(
        fixture.id(),
        fixture.universe_id(),
        occupied_addresses,
        fixture_by_address_index,
    )
}

/// Helper function to call [`validate_fixture_address_with_params()`].
fn validate_fixture_address_change(
    fixture: &Fixture,
    change: &FixtureChange,
    occupied_addresses: &[DmxAddress],
    fixture_by_address_index: &HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
) -> Result<(), ValidateError> {
    match change {
        FixtureChange::Universe(u_id) => validate_fixture_address_with_params(
            fixture.id(),
            *u_id,
            occupied_addresses,
            fixture_by_address_index,
        ),
        FixtureChange::Address(_) | FixtureChange::Mode(_) => validate_fixture_address_with_params(
            fixture.id(),
            fixture.universe_id(),
            occupied_addresses,
            fixture_by_address_index,
        ),
        _ => Ok(()),
    }
}

/// Actual validation.
fn validate_fixture_address_with_params(
    fixture_id: FixtureId,
    universe_id: UniverseId,
    occupied_addresses: &[DmxAddress],
    fixture_by_address_index: &HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
) -> Result<(), ValidateError> {
    let mut conflicts = Vec::new();

    for (new_offset, new_adr) in occupied_addresses.iter().enumerate() {
        if let Some((existing_fixture_id, offset)) =
            fixture_by_address_index.get(&(universe_id, *new_adr))
        {
            if *existing_fixture_id == fixture_id {
                continue;
            }
            conflicts.push(AddressConflictedError {
                address: *new_adr,
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
