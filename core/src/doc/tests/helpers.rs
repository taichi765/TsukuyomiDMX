use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    doc::{Doc, DocEffect, DocState, FakeFixtureDefRegistry},
    fixture::{Fixture, MergeMode},
    fixture_def::{Capability, CapabilityInner, ChannelDef, FixtureDef, FixtureDefId, FixtureMode},
    universe::{DmxAddress, UniverseId},
};

pub(crate) fn make_doc_state_with_simple_def() -> (Arc<DocState>, FixtureDefId) {
    let mut def_rg = FakeFixtureDefRegistry::new();
    let def = {
        let mut def = FixtureDef::new("Test Manufacturer", "Test Model");
        def.insert_channel(
            "Dimmer",
            ChannelDef::new(
                MergeMode::HTP,
                Capability::Single(CapabilityInner::Intensity),
            ),
        );
        def.insert_channel(
            "Red",
            ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Red)),
        );
        def.insert_channel(
            "Green",
            ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Green)),
        );
        def.insert_channel(
            "Blue",
            ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Blue)),
        );
        def.insert_mode(
            "Mode 1",
            FixtureMode::new(
                vec![("Dimmer", 0), ("Red", 1), ("Green", 2), ("Blue", 3)]
                    .into_iter()
                    .map(|(s, n)| (s.to_string(), n)),
            )
            .unwrap(),
        );
        def
    };
    let def_id = def.id().to_owned();
    def_rg.insert(def_id.clone(), def);
    (Arc::new(DocState::new(Box::new(def_rg))), def_id)
}

pub(crate) fn make_simple_fixture(def_id: FixtureDefId) -> Fixture {
    Fixture::new(
        "Test Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        def_id.clone(),
        "Mode 1",
        0.,
        0.,
    )
}

/// Creates a new Doc with an event collector already subscribed.
/// Returns the Doc and the collected events for verification.
pub(crate) fn make_doc_with_observer() -> (Doc, Rc<RefCell<Vec<DocEffect>>>) {
    let def_rg = FakeFixtureDefRegistry::new(); // TODO: 呼び出し側でdefを追加できない
    let mut doc = Doc::new_with_def_registry(Box::new(def_rg));
    let events = Rc::new(RefCell::new(Vec::new()));
    let events_clone = Rc::clone(&events);
    doc.subscribe(Box::new(move |effect| {
        events_clone.borrow_mut().push(effect.clone());
    }));
    (doc, events)
}
