use super::helpers::make_doc_state_with_simple_def;
use crate::{
    doc::{AddFixtureCommand, DocCommand, DocEffect, RemoveFixtureCommand, UpdateFixtureCommand},
    fixture::{Fixture, FixtureChange},
    fixture_def::AddressIter,
    prelude::{DmxAddress, UniverseId},
};
use assert_matches::assert_matches;

#[test]
fn add_fixture_works() {
    let (state, def_id) = make_doc_state_with_simple_def();

    // TODO: DRY
    let fxt = Fixture::new(
        "Test Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        def_id.clone(),
        "4 Channel",
        0.,
        0.,
    );
    let fxt_id = fxt.id();
    let occupied_addresses = state.with_fixture_defs(|it| {
        it.get(&def_id)
            .unwrap()
            .mode(fxt.fixture_mode())
            .unwrap()
            .occupied_addresses(fxt.universe_id(), fxt.address())
    });

    let cmd = Box::new(AddFixtureCommand::new(fxt, occupied_addresses));

    let (rev_cmd, effect) = cmd.clone().apply(&state);

    assert!(state.with_fixtures(|it| it.contains_key(&fxt_id)));
    assert!(state.with_address_index(|index| {
        (1..=4)
            .map(|n| DmxAddress::new(n).unwrap())
            .all(|adr| index.contains_key(&(UniverseId::new(0), adr)))
    }));
    assert_matches!(effect, DocEffect::FixtureAdded(id) if id == fxt_id);

    let (rev_rev_cmd, _) = rev_cmd.apply(&state);
    let rev_rev_cmd = rev_rev_cmd
        .as_any()
        .downcast_ref::<AddFixtureCommand<AddressIter>>()
        .unwrap();
    assert_eq!(cmd.as_ref(), rev_rev_cmd);
}

#[test]
fn update_fixture_rename_works() {
    let (state, def_id) = make_doc_state_with_simple_def();

    let fxt = Fixture::new(
        "Test Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        def_id.clone(),
        "4 Channel",
        0.,
        0.,
    );
    let fxt_id = fxt.id();
    let occupied_addresses = state.with_fixture_defs(|it| {
        it.get(&def_id)
            .unwrap()
            .mode(fxt.fixture_mode())
            .unwrap()
            .occupied_addresses(fxt.universe_id(), fxt.address())
    });

    let _ = Box::new(AddFixtureCommand::new(fxt, occupied_addresses.clone())).apply(&state);

    let cmd = Box::new(UpdateFixtureCommand::new(
        fxt_id,
        FixtureChange::Rename("Renamed Fixture".to_string()),
        occupied_addresses.clone(),
        occupied_addresses,
    ));
    let (rev_cmd, effect) = cmd.clone().apply(&state);

    assert!(state.with_fixtures(|it| it.get(&fxt_id).unwrap().name() == "Renamed Fixture"));
    assert_matches!(effect, DocEffect::FixtureUpdated(id) if id==fxt_id);

    let (rev_rev_cmd, _) = rev_cmd.apply(&state);
    let rev_rev_cmd = rev_rev_cmd
        .as_any()
        .downcast_ref::<UpdateFixtureCommand<AddressIter>>()
        .unwrap();
    assert_eq!(cmd.as_ref(), rev_rev_cmd)
}

#[test]
fn remove_fixture_works() {
    let (state, def_id) = make_doc_state_with_simple_def();

    let fxt = Fixture::new(
        "Test Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        def_id.clone(),
        "4 Channel",
        0.,
        0.,
    );
    let fxt_id = fxt.id();
    let occupied_addresses = state.with_fixture_defs(|it| {
        it.get(&def_id)
            .unwrap()
            .mode(fxt.fixture_mode())
            .unwrap()
            .occupied_addresses(fxt.universe_id(), fxt.address())
    });

    let _ = Box::new(AddFixtureCommand::new(fxt, occupied_addresses.clone())).apply(&state);

    let cmd = Box::new(RemoveFixtureCommand::new(fxt_id));
    let (rev_cmd, effect) = cmd.clone().apply(&state);

    assert!(state.with_fixtures(|it| it.get(&fxt_id).is_none()));
    assert!(state.with_address_index(|index| {
        (1..=4)
            .map(|n| DmxAddress::new(n).unwrap())
            .all(|adr| !index.contains_key(&(UniverseId::new(0), adr)))
    }));
    assert_matches!(effect, DocEffect::FixtureRemoved(id) if id == fxt_id);

    let (rev_rev_cmd, _) = rev_cmd.apply(&state);
    let rev_rev_cmd = rev_rev_cmd
        .as_any()
        .downcast_ref::<RemoveFixtureCommand>()
        .unwrap();
    assert_eq!(cmd.as_ref(), rev_rev_cmd);
}
