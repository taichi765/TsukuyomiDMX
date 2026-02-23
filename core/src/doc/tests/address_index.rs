#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn get_fixture_by_address_returns_fixture_and_offset_for_each_occupied_address() {
    /*let mut doc = DocStore::new();

    // Prepare def with two channels and ModeA
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Fixture at base address 50 using ModeA
    let base_addr = 50;
    let fxt = make_fixture(
        "FxtTwo",
        def_id,
        uni_id,
        DmxAddress::new(base_addr).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();

    doc.add_fixture(fxt).unwrap();

    // Expect address index contains base+0 offset 0
    let got0 = doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr).unwrap());
    assert!(got0.is_some());
    let (id, offset) = got0.unwrap();
    assert_eq!(*id, fxt_id);
    assert_eq!(*offset, 0);

    // Expect address index contains base+1 offset 1
    let got1 = doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr + 1).unwrap());
    assert!(got1.is_some());

    let (id, offset) = got1.unwrap();
    assert_eq!(*id, fxt_id);
    assert_eq!(*offset, 1);

    // An address not occupied by the fixture should be None
    let got2 = doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr + 2).unwrap());
    assert!(got2.is_none());*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn address_index_is_cleared_when_fixture_is_removed() {
    /*let mut doc = DocStore::new();

    // Prepare and insert def
    let def = make_def_with_two_channels();
    let def_id = def.id();
    doc.insert_fixture_def(def);

    // Universe
    let uni_id = UniverseId::new(1);
    doc.add_universe(uni_id);

    // Insert fixture
    let base_addr = 20;
    let fxt = make_fixture(
        "FxRemove",
        def_id,
        uni_id,
        DmxAddress::new(base_addr).unwrap(),
        "ModeA",
    );
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).unwrap();

    // Sanity: index has entries
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr).unwrap())
            .is_some()
    );
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr + 1).unwrap())
            .is_some()
    );

    // Remove fixture -> Ok(Some(...))
    let removed = doc.remove_fixture(&fxt_id).unwrap();
    assert!(removed.is_some());

    // Index should no longer contain the addresses
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr).unwrap())
            .is_none()
    );
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base_addr + 1).unwrap())
            .is_none()
    );*/
}

#[test]
#[ignore = "address_indexはDocの外に移動する予定"]
fn address_index_is_not_populated_when_add_fixture_errors() {
    /*let mut doc = DocStore::new();

    // Universe present, but no fixture def inserted
    let uni_id = UniverseId::new(2);
    doc.add_universe(uni_id);

    // Create a fixture referencing a non-existent FixtureDef
    let dummy_def_id = FixtureDef::new("Dummy", "Missing").id();
    let base = 100;
    let fxt = Fixture::new(
        "FxErr",
        uni_id,
        DmxAddress::new(base).unwrap(),
        dummy_def_id,
        "ModeA".into(),
        0.,
        0.,
    );

    let fxt_id = fxt.id();

    // Insertion should error with FixtureDefNotFound
    let err = doc.add_fixture(fxt).expect_err("expected add error");
    assert!(matches!(
        err,
        FixtureAddError::FixtureDefNotFound(FixtureDefNotFound{
            fixture_id:f_id,fixture_def_id:d_id
        }) if f_id==fxt_id&&d_id==dummy_def_id
    ));

    // Address index should remain empty for the universe
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base).unwrap())
            .is_none()
    );
    assert!(
        doc.get_fixture_by_address(&uni_id, DmxAddress::new(base + 1).unwrap())
            .is_none()
    );*/
}
