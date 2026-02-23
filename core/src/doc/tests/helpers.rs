use std::{cell::RefCell, rc::Rc};

use crate::{
    doc::{Doc, DocEffect},
    fixture::{Fixture, MergeMode},
    fixture_def::{ChannelDef, ChannelKind, FixtureDef, FixtureDefId, FixtureMode},
    functions::{FunctionData, StaticSceneData},
    universe::{DmxAddress, UniverseId},
};

/// Creates a new Doc with an event collector already subscribed.
/// Returns the Doc and the collected events for verification.
pub(crate) fn make_doc_with_observer() -> (Doc, Rc<RefCell<Vec<DocEffect>>>) {
    let mut doc = Doc::new_with_def_registry();
    let events = Rc::new(RefCell::new(Vec::new()));
    let events_clone = Rc::clone(&events);
    doc.subscribe(Box::new(move |effect| {
        events_clone.borrow_mut().push(effect.clone());
    }));
    (doc, events)
}

/// Build a minimal FixtureDef with a single mode and dummy channels + single named channel.
/// - manufacturer is fixed to "TestMfr"
/// - model is provided via `model`
/// - the mode `mode_name` contains `channel_name` at `channel_offset` with `merge_mode`
/// - channel provided via `channel_name` is created at `channel_offset`.
///     Other channels is created with name `Dummy{offset}`.
pub(crate) fn make_fixture_def_with_mode(
    model: &str,
    mode_name: &str,
    channel_name: &str,
    channel_offset: usize,
    merge_mode: MergeMode,
    kind: ChannelKind,
) -> FixtureDef {
    let mut def = FixtureDef::new("TestMfr".to_string(), model.to_string());

    let mut channel_order = Vec::new();

    def.insert_channel(
        String::from(channel_name),
        ChannelDef::new(merge_mode, kind),
    );
    channel_order.push((channel_name.to_string(), channel_offset));

    (0..channel_offset).for_each(|ch| {
        let ch_name = format!("Dummy{}", ch);
        def.insert_channel(
            ch_name.clone(),
            ChannelDef::new(MergeMode::HTP, ChannelKind::Custom),
        );
        channel_order.push((ch_name, ch));
    });

    let mode = FixtureMode::new(channel_order.into_iter()).unwrap();
    def.insert_mode(String::from(mode_name), mode);

    def
}

pub(crate) fn make_def_with_two_channels() -> FixtureDef {
    // Manufacturer/Model arbitrary for test
    let mut def = FixtureDef::new("TestMfr", "ModelDual");

    // Insert two channel templates: Dimmer (offset 0) and Color (offset 3)
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(crate::fixture::MergeMode::LTP, ChannelKind::Dimmer),
    );
    def.insert_channel(
        "Color",
        ChannelDef::new(crate::fixture::MergeMode::HTP, ChannelKind::Red),
    );

    // Mode order specifies offsets
    let order = vec![("Dimmer".to_string(), 0), ("Color".to_string(), 1)];
    let mode = FixtureMode::new(order.into_iter()).unwrap();
    def.insert_mode("ModeA", mode);

    def
}

/// Build a Fixture that references a given FixtureDef and mode.
pub(crate) fn make_fixture(
    name: &str,
    fixture_def_id: FixtureDefId,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_mode: &str,
) -> Fixture {
    Fixture::new(
        name,
        universe_id,
        address,
        fixture_def_id,
        String::from(fixture_mode),
        0.,
        0.,
    )
}

/// Build a simple FunctionData (StaticScene) with the given name.
pub(crate) fn make_function(name: &str) -> FunctionData {
    FunctionData::StaticScene(StaticSceneData::new(name))
}
