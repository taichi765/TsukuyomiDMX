#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_returns_none_for_empty_universe() {
    /*let mut doc = DocStore::new();

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // No fixtures in universe -> None
    let result = doc.current_max_address(uni_id);
    assert!(result.is_none());*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_returns_none_for_nonexistent_universe() {
    /*let doc = DocStore::new();

    // Universe not added -> None
    let uni_id = UniverseId::new(99);
    let result = doc.current_max_address(uni_id);
    assert!(result.is_none());*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_returns_last_occupied_address_for_single_fixture() {
    /*let mut doc = DocStore::new();

    // Prepare def with two channels (offsets 0 and 1)
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Fixture at base address 50 using ModeA (occupies 50 and 51)
    let base_addr = 50;
    let fxt = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(base_addr).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt).unwrap();

    let result = doc.current_max_address(uni_id);
    assert!(result.is_some());
    // Two channels at offsets 0 and 1 -> max occupied address is 51
    assert_eq!(result.unwrap().value(), base_addr + 1);*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_returns_highest_among_multiple_fixtures() {
    /*let mut doc = DocStore::new();

    // Prepare def with two channels (offsets 0 and 1)
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // First fixture at base address 10 (occupies 10 and 11)
    let fxt1 = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt1).unwrap();

    // Second fixture at base address 100 (occupies 100 and 101)
    let fxt2 = make_fixture(
        "Fxt2",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt2).unwrap();

    // Third fixture at base address 50 (occupies 50 and 51)
    let fxt3 = make_fixture(
        "Fxt3",
        def_id,
        uni_id,
        DmxAddress::new(50).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt3).unwrap();

    let result = doc.current_max_address(uni_id);
    assert!(result.is_some());
    // Highest base address is 100, with 2 channels -> max occupied is 101
    assert_eq!(result.unwrap().value(), 101);*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_considers_only_specified_universe() {
    /*let mut doc = DocStore::new();

    // Prepare def with two channels
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni1 = UniverseId::new(1);
    let uni2 = UniverseId::new(2);
    doc.add_universe(uni1);
    doc.add_universe(uni2);

    // Fixture in universe 1 at address 200 (occupies 200 and 201)
    let fxt1 = make_fixture(
        "FxtUni1",
        def_id,
        uni1,
        DmxAddress::new(200).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt1).unwrap();

    // Fixture in universe 2 at address 50 (occupies 50 and 51)
    let fxt2 = make_fixture(
        "FxtUni2",
        def_id,
        uni2,
        DmxAddress::new(50).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt2).unwrap();

    // Check universe 1
    let result1 = doc.current_max_address(uni1);
    assert!(result1.is_some());
    assert_eq!(result1.unwrap().value(), 201);

    // Check universe 2
    let result2 = doc.current_max_address(uni2);
    assert!(result2.is_some());
    assert_eq!(result2.unwrap().value(), 51);*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_updates_after_fixture_removal() {
    /*let mut doc = DocStore::new();

    // Prepare def with two channels
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // First fixture at base address 10 (occupies 10 and 11)
    let fxt1 = make_fixture(
        "Fxt1",
        def_id,
        uni_id,
        DmxAddress::new(10).unwrap(),
        "ModeA",
    );
    let fxt1_id = fxt1.id();
    doc.add_fixture(fxt1).unwrap();

    // Second fixture at base address 100 (occupies 100 and 101)
    let fxt2 = make_fixture(
        "Fxt2",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    let fxt2_id = fxt2.id();
    doc.add_fixture(fxt2).unwrap();

    // Before removal: max is 101
    assert_eq!(doc.current_max_address(uni_id).unwrap().value(), 101);

    // Remove the fixture with highest address
    doc.remove_fixture(&fxt2_id).unwrap();

    // After removal: max should be 11
    assert_eq!(doc.current_max_address(uni_id).unwrap().value(), 11);

    // Remove the remaining fixture
    doc.remove_fixture(&fxt1_id).unwrap();

    // After all removed: None
    assert!(doc.current_max_address(uni_id).is_none());*/
}

#[test]
#[ignore = "DocState::current_max_address is not implemented yet (todo!)"]
fn current_max_address_with_single_channel_fixture() {
    /*let mut doc = DocStore::new();

    // Prepare def with single channel at offset 0
    let def = make_fixture_def_with_mode(
        "ModelSingle",
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

    // Fixture at base address 100 (occupies only 100)
    let fxt = make_fixture(
        "FxtSingle",
        def_id,
        uni_id,
        DmxAddress::new(100).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt).unwrap();

    let result = doc.current_max_address(uni_id);
    assert!(result.is_some());
    // Single channel at offset 0 -> max occupied address is 100
    assert_eq!(result.unwrap().value(), 100);*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn current_max_address_at_dmx_boundary() {
    /*let mut doc = DocStore::new();

    // Prepare def with single channel at offset 0
    let def = make_fixture_def_with_mode(
        "ModelBoundary",
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

    // Fixture at address 512 (max valid DMX address)
    let fxt = make_fixture(
        "FxtMax",
        def_id,
        uni_id,
        DmxAddress::new(512).unwrap(),
        "ModeA",
    );
    doc.add_fixture(fxt).unwrap();

    let result = doc.current_max_address(uni_id);
    assert!(result.is_some());
    assert_eq!(result.unwrap().value(), 512);*/
}
