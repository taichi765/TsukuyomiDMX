use std::sync::Arc;

use assert_matches::assert_matches;

use super::helpers::{make_doc_state_with_simple_def, make_simple_fixture};
use crate::{
    doc::{
        DocState, DocStateView, FakeFixtureDefRegistry, FixtureAddError, FixtureDefNotFoundError,
        ModeNotFoundError, decider,
    },
    fixture::Fixture,
    prelude::{DmxAddress, FixtureDefId, UniverseId},
};

#[test]
fn add_fixture_returns_ok() {
    let (state, def_id) = make_doc_state_with_simple_def();
    let fxt = make_simple_fixture(def_id);
    let fxt_id = fxt.id();

    let cmd =
        decider::add_fixture(DocStateView(Arc::clone(&state)), fxt).expect("should return Ok()");

    assert_eq!(cmd.fixture().id(), fxt_id);
    let mut addresses = cmd.occupied_addresses().clone();
    assert_eq!(addresses.clone().count(), 4); // TODO: remove magic number
    assert_eq!(
        addresses.next().unwrap(),
        (UniverseId::new(0), DmxAddress::MIN)
    );
}

#[test]
fn add_fixture_returns_def_not_found_error() {
    let def_rg = Box::new(FakeFixtureDefRegistry::new());
    let state = Arc::new(DocState::new(def_rg));

    let def_id = FixtureDefId::new_invalid();
    let fxt = make_simple_fixture(def_id.clone());
    let fxt_id = fxt.id();

    let res = decider::add_fixture(DocStateView(Arc::clone(&state)), fxt);
    assert_matches!(res, Err(FixtureAddError::FixtureDefNotFound(
        FixtureDefNotFoundError{fixture_id:f_id, fixture_def_id:d_id,source:_}
    )) if f_id == fxt_id&&d_id == def_id);
}

#[test]
fn add_fixture_returns_mode_not_found_error() {
    let (state, def_id) = make_doc_state_with_simple_def();

    let fxt = Fixture::new(
        "Test Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        def_id.clone(),
        "Dummy Mode",
        0.,
        0.,
    );

    let res = decider::add_fixture(DocStateView(Arc::clone(&state)), fxt);
    assert_matches!(res, Err(FixtureAddError::ModeNotFound(ModeNotFoundError{
        fixture_def: d_id,
        mode
    })) if d_id == def_id&&mode == "Dummy Mode".to_string());
}

#[test]
fn add_fixture_returns_address_validate_error() {}

#[test]
fn add_fixture_returns_already_exists_error() {}
