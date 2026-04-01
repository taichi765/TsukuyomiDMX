use super::helpers::{make_doc_handle_with_observer, make_fixture, make_fixture_def_with_mode};
use crate::doc::{
    DocEvent, DocStore, FixtureAddError, FixtureDefNotFound, FixtureNotFound, FixtureRemoveError,
    ModeNotFound, ResolveError,
};
use crate::fixture_def::{ChannelKind, FixtureDefId};
use crate::{
    fixture::{Fixture, MergeMode},
    universe::{DmxAddress, UniverseId},
};

/* ==================== DocStore direct tests ==================== */

#[test]
fn add_fixture_then_update_returns_old_value() {
    let mut doc = DocStore::new();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // create fixture
    let fxt = make_fixture("Fxt1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();

    // add -> Ok(())
    doc.add_fixture(fxt.clone())
        .expect("fixture add should succeed");

    // update with same id -> Ok(previous)
    let old = doc
        .update_fixture(fxt)
        .expect("fixture update should succeed");
    assert_eq!(old.id(), fxt_id);
}

#[test]
fn add_fixture_errors_when_fixture_def_missing() {
    let mut doc = DocStore::new();

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Create a fixture referencing a non-existent FixtureDef
    let missing_def = FixtureDefId::new();
    let fxt = Fixture::new(
        "FxMissingDef",
        uni_id,
        DmxAddress::new(1).expect("valid address"),
        missing_def,
        String::from("ModeA"),
        0.,
        0.,
    );

    let err = doc
        .add_fixture(fxt)
        .err()
        .expect("expected add_fixture error due to missing fixture def");
    assert!(matches!(
        err,
        FixtureAddError::FixtureDefNotFound(FixtureDefNotFound { fixture_id: _, fixture_def_id })
        if fixture_def_id == missing_def
    ));
}

#[test]
fn add_fixture_errors_when_mode_not_found_in_def() {
    let mut doc = DocStore::new();

    // Prepare a def with only "ModeB"
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeB",
        "Dimmer",
        1,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Fixture requests "ModeA" which is not present
    let fxt = make_fixture(
        "FxWrongMode",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );

    let err = doc
        .add_fixture(fxt)
        .err()
        .expect("expected error due to mode not found");
    assert!(matches!(
        err,
        FixtureAddError::ModeNotFound (ModeNotFound{ fixture_def, mode })
        if fixture_def == def_id && mode == "ModeA"
    ));
}

#[test]
fn add_fixture_errors_when_address_validation_fails_due_to_overlap() {
    let mut doc = DocStore::new();

    // Prepare a def with "ModeA" and a single channel at offset 0 (occupies base address)
    let def = make_fixture_def_with_mode(
        "ModelOverlap",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe exists
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // First fixture at address 100
    let fxt1 = make_fixture(
        "Fx1",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    let fxt1_id = fxt1.id();
    doc.add_fixture(fxt1)
        .expect("first fixture addition should succeed");
    assert!(doc.get_fixture(&fxt1_id).is_some());

    // Second fixture tries to occupy the same address -> should fail validation
    let fxt2 = make_fixture(
        "Fx2",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    let err = doc
        .add_fixture(fxt2)
        .err()
        .expect("expected error due to address overlap");
    // Only check that this maps to AddressValidateError (inner type opaque here)
    assert!(matches!(err, FixtureAddError::AddressValidateError(_)));
}

#[test]
fn remove_fixture_returns_some_then_none() {
    let mut doc = DocStore::new();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // insert fixture
    let fxt = make_fixture(
        "Fxt2",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).expect("should work");

    // remove once -> Ok(Some(...))
    let removed = doc
        .remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id(), fxt_id);

    // remove again -> Ok(None)
    assert!(
        doc.remove_fixture(&fxt_id)
            .expect("second fixture removal should succeed")
            .is_none()
    );
}

#[test]
fn remove_fixture_errors_when_fixture_def_missing() {
    let mut doc = DocStore::new();

    // prepare and insert fixture def + universe + fixture
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(2);
    doc.add_universe(uni_id);

    let fxt = make_fixture(
        "FxToRemove",
        def_id,
        uni_id,
        DmxAddress::new(5).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).expect("add_fixture should succeed");

    // Remove the fixture_def before removing fixture -> should error
    let _removed_def = doc.remove_fixture_def(&def_id);
    assert!(_removed_def.is_some());

    let err = doc
        .remove_fixture(&fxt_id)
        .err()
        .expect("expected remove_fixture to error due to missing fixture def");
    assert!(matches!(
        err,
        FixtureRemoveError::FixtureDefNotFound(FixtureDefNotFound { fixture_id, fixture_def_id })
        if fixture_id == fxt_id && fixture_def_id == def_id
    ));
}

#[test]
fn resolve_address_fails_after_fixture_removed() {
    let mut doc = DocStore::new();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        5,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // insert fixture
    let base = 20;
    let fxt = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(base).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).expect("add_fixture should succeed");

    // resolve works before removal
    let (resolved_uni, resolved) = doc.resolve_address(fxt_id, "Dimmer").unwrap();
    assert_eq!(resolved_uni, uni_id);
    assert_eq!(resolved.address.value(), base + 5);

    // remove and then resolution should fail
    doc.remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");
    let err = doc.resolve_address(fxt_id, "Dimmer").err().unwrap();
    assert!(matches!(err, ResolveError::FixtureNotFound(FixtureNotFound(id)) if id == fxt_id));
}

/* ==================== Doc event notification tests ==================== */

#[test]
fn doc_handle_add_and_update_fixture_emit_events() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    handle.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // create and add fixture
    let fxt = make_fixture("Fxt1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();

    handle
        .add_fixture(fxt.clone())
        .expect("fixture add should succeed");

    // FixtureAdded event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureAdded(id) if *id == fxt_id))
        );
    }

    // update emits FixtureUpdated
    handle
        .update_fixture(fxt)
        .expect("fixture update should succeed");

    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureUpdated(id) if *id == fxt_id))
        );
    }
}

#[test]
fn doc_handle_add_fixture_does_not_emit_event_on_error() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // Universe exists but no fixture def
    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // Create a fixture referencing a non-existent FixtureDef
    let missing_def = FixtureDefId::new();
    let fxt = Fixture::new(
        "FxMissingDef",
        uni_id,
        DmxAddress::new(1).expect("valid address"),
        missing_def,
        String::from("ModeA"),
        0.,
        0.,
    );
    let fxt_id = fxt.id();

    let err = handle.add_fixture(fxt);
    assert!(err.is_err());

    // No FixtureAdded event should be emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            !obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureAdded(id) if *id == fxt_id))
        );
    }
}

#[test]
fn doc_handle_update_fixture_does_not_emit_event_on_eroor() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // Universe exists
    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // Prepare a def with only "ModeB"
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeB",
        "Dimmer",
        1,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    handle.insert_fixture_def(def);

    // Fixture requests "ModeA" which is not present
    let fxt = Fixture::new(
        "FxMissingDef",
        uni_id,
        DmxAddress::new(1).expect("valid address"),
        def_id,
        String::from("ModeA"),
        0.,
        0.,
    );
    let fxt_id = fxt.id();

    let err = handle.add_fixture(fxt);
    assert!(err.is_err());

    // No FixtureAdded event should be emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            !obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureAdded(id) if *id == fxt_id))
        );
    }
}

#[test]
fn doc_handle_remove_fixture_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // prepare dependencies: fixture def + universe
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    handle.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    handle.add_universe(uni_id);

    // insert fixture
    let fxt = make_fixture(
        "Fxt2",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    handle.add_fixture(fxt).expect("should work");

    // remove once -> Ok(Some(...))
    let removed = handle
        .remove_fixture(&fxt_id)
        .expect("fixture removal should succeed");
    assert!(removed.is_some());

    // event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureRemoved(id) if *id == fxt_id))
        );
    }
}

#[test]
fn doc_handle_remove_fixture_does_not_emit_event_on_error() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // prepare and insert fixture def + universe + fixture
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        0,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();
    handle.insert_fixture_def(def);

    let uni_id = UniverseId::new(2);
    handle.add_universe(uni_id);

    let fxt = make_fixture(
        "FxToRemove",
        def_id,
        uni_id,
        DmxAddress::new(5).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    handle
        .add_fixture(fxt)
        .expect("fixture insert should succeed");

    // Clear previous events
    observer.write().unwrap().events.clear();

    // Remove the fixture_def before removing fixture -> should error
    handle.remove_fixture_def(&def_id);

    let err = handle.remove_fixture(&fxt_id);
    assert!(err.is_err());

    // No FixtureRemoved event should be emitted for the failed removal
    {
        let obs = observer.read().unwrap();
        assert!(
            !obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureRemoved(id) if *id == fxt_id))
        );
    }
}
