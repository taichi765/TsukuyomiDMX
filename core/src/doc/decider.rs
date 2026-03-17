//! Validates command and returns [`DocEvent`] or an error.
//! Similar to event sourcing's decider.

use super::errors::*;
use super::{DocEffect, DocStateView};
use crate::doc::commands::{AddFixtureCommand, RemoveFixtureCommand, UpdateFixtureCommand};
use crate::doc::state::AddressIndex;
use crate::fixture::FixtureChange;
use crate::functions::FunctionData;
use crate::prelude::*;

pub(super) fn add_fixture(
    state: DocStateView,
    fixture: Fixture,
) -> Result<AddFixtureCommand<impl Iterator<Item = (UniverseId, DmxAddress)>>, FixtureAddError> {
    if state.with_fixtures(|it| it.contains_key(&fixture.id())) {
        return Err(FixtureAddError::FixtureAlreadyExists(fixture.id()));
    }

    let def_id = fixture.fixture_def();
    let occupied_addresses = state.with_fixture_defs(|defs| {
        let def = defs.get(&def_id).map_err(|e| {
            FixtureAddError::FixtureDefNotFound(FixtureDefNotFound {
                fixture_id: fixture.id(),
                fixture_def_id: def_id.clone(),
                source: e,
            })
        })?;
        let mode = def
            .mode(fixture.fixture_mode())
            .ok_or(FixtureAddError::ModeNotFound(ModeNotFound {
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
) -> Result<UpdateFixtureCommand, FixtureUpdateError> {
    state.with_fixtures_and_defs(|fxts, defs| -> Result<(), FixtureUpdateError> {
        let fxt = fxts.get(&id).ok_or(FixtureNotFound(id))?;
        let def = defs.get(&fxt.fixture_def()).unwrap();
        let occupied_addresses = compute_occupied_addresses(fxt, def, &change)?;

        state.with_address_index(|index| {
            validate_fixture_address_change(fxt, &change, occupied_addresses, index)
                .map_err(|e| FixtureUpdateError::AddressValidateError(e))
        })
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

pub(super) fn add_function(_state: DocStateView, _value: FunctionData) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn update_function(_state: DocStateView, _new: FunctionData) -> Result<DocEffect, ()> {
    todo!()
}

pub(super) fn remove_function(_state: DocStateView, _id: &FunctionId) -> Result<DocEffect, ()> {
    todo!()
}

/// changeを適用したあとのoccupied_addressを計算する
fn compute_occupied_addresses(
    fixture: &Fixture,
    def: &FixtureDef,
    change: &FixtureChange,
) -> Result<impl Iterator<Item = (UniverseId, DmxAddress)>, ModeNotFound> {
    match change {
        FixtureChange::Mode(mode_name) => {
            let mode = def.mode(mode_name).ok_or(ModeNotFound {
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

/// Helper function to call [`validate_fixture_address()`].
///
/// If change doesn't affect to the address, it does nothing and `Ok(())` is returned.
fn validate_fixture_address_change(
    fixture: &Fixture,
    change: &FixtureChange,
    occupied_addresses: impl Iterator<Item = (UniverseId, DmxAddress)>,
    address_index: &AddressIndex,
) -> Result<(), ValidateError> {
    match change {
        FixtureChange::Address(_) | FixtureChange::Mode(_) | FixtureChange::Universe(_) => {
            validate_fixture_address(fixture.id(), occupied_addresses, address_index)
        }
        _ => Ok(()),
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
