use super::helpers::{make_fixture, make_fixture_def_with_mode};
use crate::{
    doc::{DocState, FixtureNotFound, ResolveError},
    fixture::{FixtureId, MergeMode},
    fixture_def::{ChannelKind, FixtureDef, FixtureMode},
    universe::{DmxAddress, UniverseId},
};

// Note: These tests are ignored because DocState::resolve_address is todo!() in the new architecture.
// Tests should be rewritten when resolve_address is implemented.

#[test]
#[ignore = "DocState::resolve_address is not implemented yet (todo!)"]
fn resolve_success_single_channel() {
    // This test requires DocState::resolve_address which is currently todo!().
}

#[test]
#[ignore = "DocState::resolve_address is not implemented yet (todo!)"]
fn resolve_error_fixture_not_found() {
    // This test requires DocState::resolve_address which is currently todo!().
}

#[test]
#[ignore = "DocState::resolve_address is not implemented yet (todo!)"]
fn resolve_error_channel_not_found() {
    // This test requires DocState::resolve_address which is currently todo!().
}
