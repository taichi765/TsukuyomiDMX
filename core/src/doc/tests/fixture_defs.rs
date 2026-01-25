use super::helpers::{make_doc_handle_with_observer, make_fixture, make_fixture_def_with_mode};
use crate::{
    doc::{DocEvent, DocState, FixtureDefNotFound, ResolveError},
    fixture::MergeMode,
    fixture_def::ChannelKind,
    universe::{DmxAddress, UniverseId},
};

/* ==================== DocStore direct tests ==================== */

#[test]
fn insert_fixture_def_allows_resolution() {
    let mut doc = DocState::new();

    // prepare a fixture def with one mode and one channel
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        5,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();

    // insert
    let old = doc.insert_fixture_def(def);
    assert!(old.is_none());

    // add universe and fixture that uses the def/mode, then resolve address
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    let base_addr = 10;
    let fxt = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(base_addr).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).expect("should work");

    let (resolved_uni, resolved_addr) = doc.resolve_address(fxt_id, "Dimmer").unwrap();
    assert_eq!(resolved_uni, uni_id);
    assert_eq!(resolved_addr.address.value(), base_addr + 5);
    matches!(resolved_addr.merge_mode, MergeMode::LTP);
}

#[test]
fn remove_fixture_def_breaks_resolution() {
    let mut doc = DocState::new();

    // prepare and insert fixture def
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

    // add universe and fixture referencing the def
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    let fxt = make_fixture("Fxt1", def_id, uni_id, DmxAddress::new(1).unwrap(), "ModeA");
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).expect("should work");

    // now remove fixture def
    let removed = doc.remove_fixture_def(&def_id);
    assert!(removed.is_some());

    // resolution should now fail with FixtureDefNotFound
    let err = doc.resolve_address(fxt_id, "Dimmer").err().unwrap();
    assert!(matches!(
        err,
        ResolveError::FixtureDefNotFound (FixtureDefNotFound{
            fixture_id: _,
            fixture_def_id
        }) if fixture_def_id == def_id
    ));
}

#[test]
fn remove_nonexistent_fixture_def_returns_none() {
    let mut doc = DocState::new();

    // random UUID via a dummy def: create and drop to get a valid id, then remove twice
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Ch",
        1,
        MergeMode::HTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();

    // first insert then remove
    doc.insert_fixture_def(def);
    assert!(doc.remove_fixture_def(&def_id).is_some());

    // removing again should return None
    assert!(doc.remove_fixture_def(&def_id).is_none());
}

/* ==================== DocHandle event notification tests ==================== */

#[test]
fn doc_handle_insert_fixture_def_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // prepare a fixture def
    let def = make_fixture_def_with_mode(
        "ModelX",
        "ModeA",
        "Dimmer",
        5,
        MergeMode::LTP,
        ChannelKind::Dimmer,
    );
    let def_id = def.id();

    // insert
    let old = handle.insert_fixture_def(def);
    assert!(old.is_none());

    // event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureDefAdded(id) if *id == def_id))
        );
    }
}

#[test]
fn doc_handle_remove_fixture_def_emits_event() {
    let (handle, _doc_store, observer) = make_doc_handle_with_observer();

    // prepare and insert fixture def
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

    // remove fixture def
    let removed = handle.remove_fixture_def(&def_id);
    assert!(removed.is_some());

    // event emitted
    {
        let obs = observer.read().unwrap();
        assert!(
            obs.events
                .iter()
                .any(|e| matches!(e, DocEvent::FixtureDefRemoved(id) if *id == def_id))
        );
    }
}
