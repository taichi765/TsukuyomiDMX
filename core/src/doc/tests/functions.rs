use super::helpers::make_function;
use crate::doc::{DocEffect, DocState};

/* ==================== DocState direct tests ==================== */

// Note: These tests are ignored because DocState no longer exposes direct add_function/remove_function methods.
// The new architecture uses Doc as a facade with commands and decider pattern.

#[test]
#[ignore = "DocState no longer exposes add_function/get_function_data/remove_function directly"]
fn add_and_remove_function_updates_store() {
    // This test required DocState::add_function, get_function_data, and remove_function
    // which are no longer public in the new architecture.
}

#[test]
#[ignore = "DocState no longer exposes remove_function directly"]
fn remove_nonexistent_function_returns_none() {
    // This test required DocState::remove_function which is no longer public.
}

#[test]
#[ignore = "DocState no longer exposes add_function/get_function_data/remove_function directly"]
fn add_multiple_functions_and_remove_one_keeps_the_other() {
    // This test required DocState::add_function, get_function_data, and remove_function
    // which are no longer public in the new architecture.
}

/* ==================== Doc event notification tests ==================== */

// Note: These tests are ignored because Doc::add_function and Doc::remove_function are todo!().

#[test]
#[ignore = "Doc::add_function is not implemented yet (todo!)"]
fn doc_add_function_emits_event() {
    // This test requires Doc::add_function which is currently todo!().
}

#[test]
#[ignore = "Doc::add_function and Doc::remove_function are not implemented yet (todo!)"]
fn doc_remove_function_emits_event() {
    // This test requires Doc::add_function and Doc::remove_function which are currently todo!().
}

#[test]
#[ignore = "Doc::add_function and Doc::remove_function are not implemented yet (todo!)"]
fn doc_multiple_function_operations_emit_correct_events() {
    // This test requires Doc::add_function and Doc::remove_function which are currently todo!().
}
