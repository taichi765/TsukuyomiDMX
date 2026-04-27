use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    time::Duration,
};

use tsukuyomidmx_core::{
    doc::Doc,
    effects::{Effect, EffectChange, FixtureQuery, SimpleEffectBody},
    engine::{Engine, EngineCommand},
    plugins::SpyPluginInfo,
    prelude::{DmxAddress, Fixture, FixtureDefId, UniverseId},
};

#[test]
fn engine_can_start_function() {
    /*let mut def_rg = FakeFixtureDefRegistry::new();
    let str=
    let dto: ofl_schemas::Fixture = serde_json::from_str(
        &std::fs::read_to_string("fixtures/american-dj/mega-tripar-profile-plus.json").unwrap(),
    )
    .unwrap();
    let def = FixtureDef::try_from(("american-dj".into(), dto)).unwrap();
    let def_id = def.id().to_owned();
    def_rg.insert(def_id.clone(), def);*/
    // TODO: dummyを使うべき
    let mut doc = Doc::try_new().unwrap();
    doc.add_universe();

    let fxt = Fixture::new(
        "Fixture",
        UniverseId::new(0),
        DmxAddress::MIN,
        FixtureDefId::new("uking".into(), "par-light-b262".into()),
        "7-channel",
        0.,
        0.,
    );
    //let fxt_id = fxt.id();
    doc.add_fixture(fxt).unwrap();

    let fx = Effect::new_simple("Scene 1");

    let fx_id = fx.id();
    doc.add_effect(fx).unwrap();

    let new_body = SimpleEffectBody::New {
        fixtures: FixtureQuery::from_model(
            FixtureDefId::new("uking".into(), "par-light-b262".into()),
            doc.state_view(),
        )
        .unwrap(),
        values: HashMap::from([(0, 255), (1, 200)]),
    };
    doc.update_effect(fx_id, EffectChange::Simple(new_body))
        .unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (message_tx, _message_rx) = mpsc::channel();
    let engine = Engine::new(doc.state_view(), command_rx, message_tx);
    let handle = std::thread::spawn(|| {
        engine.start_loop();
    });

    let info = Box::new(SpyPluginInfo::new(UniverseId::MIN));
    let data = Arc::clone(&info.data);
    command_tx
        .send(EngineCommand::UniverseAdded(UniverseId::new(0)))
        .unwrap();
    command_tx.send(EngineCommand::AddPlugin(info)).unwrap();
    command_tx
        .send(EngineCommand::StartFunction(fx_id))
        .unwrap();

    std::thread::sleep(Duration::from_secs(2));

    command_tx.send(EngineCommand::StopFunction).unwrap();

    command_tx.send(EngineCommand::Shutdown).unwrap();

    std::thread::sleep(Duration::from_millis(500));

    let guard = data.try_read().unwrap();
    /*let first_address: Vec<_> = guard
        .iter()
        .map(|frame| frame.as_slice()[0])
        .filter(|&v| v != 0)
        .collect();
    assert_ne!(0, first_address.len());*/
    let frame = guard[0].as_slice();
    assert_eq!(frame[0], 255);
    assert_eq!(frame[1], 200);
    handle.join().unwrap();
}
