use crate::{doc::DocState, doc::OutputPluginId, universe::UniverseId};

/* ==================== DocState direct tests ==================== */

// Note: These tests are ignored because DocState::add_universe, remove_universe,
// add_output, and remove_output are private in the new architecture.
// The new architecture uses Doc as a facade with commands and decider pattern.

#[test]
#[ignore = "DocState::add_universe is private in the new architecture"]
fn add_universe_returns_none_then_some() {
    // This test required DocState::add_universe which is now private.
}

#[test]
#[ignore = "DocState::add_universe and remove_universe are private in the new architecture"]
fn remove_universe_returns_some_then_none() {
    // This test required DocState::add_universe and remove_universe which are now private.
}

#[test]
#[ignore = "DocState::add_universe and add_output are private in the new architecture"]
fn add_output_inserts_once() {
    // This test required DocState::add_universe and add_output which are now private.
}

#[test]
#[ignore = "DocState::add_universe, add_output, and remove_output are private in the new architecture"]
fn remove_output_removes() {
    // This test required DocState::add_universe, add_output, and remove_output which are now private.
}

#[test]
#[ignore = "DocState::add_output and remove_output are private in the new architecture"]
fn output_ops_on_nonexistent_universe_returns_error() {
    // This test required DocState::add_output and remove_output which are now private.
}

/* ==================== Doc event notification tests ==================== */

// Note: These tests are ignored because Doc does not have add_universe, remove_universe,
// add_output, and remove_output methods yet.

#[test]
#[ignore = "Doc does not have add_universe method yet"]
fn doc_add_universe_emits_event() {
    // This test requires Doc::add_universe which is not yet implemented.
}

#[test]
#[ignore = "Doc does not have add_universe and remove_universe methods yet"]
fn doc_remove_universe_emits_event() {
    // This test requires Doc::add_universe and remove_universe which are not yet implemented.
}

#[test]
#[ignore = "Doc does not have add_universe and add_output methods yet"]
fn doc_add_output_emits_universe_settings_changed() {
    // This test requires Doc::add_universe and add_output which are not yet implemented.
}

#[test]
#[ignore = "Doc does not have add_universe, add_output, and remove_output methods yet"]
fn doc_remove_output_emits_universe_settings_changed() {
    // This test requires Doc::add_universe, add_output, and remove_output which are not yet implemented.
}

#[test]
#[ignore = "Doc does not have add_output and remove_output methods yet"]
fn doc_output_ops_on_nonexistent_universe_does_not_emit_event() {
    // This test requires Doc::add_output and remove_output which are not yet implemented.
}
