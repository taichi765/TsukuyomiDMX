use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

use tsukuyomidmx_core::{
    doc::Doc,
    effects::Effect,
    engine::{Engine, EngineCommand},
    plugins::{Plugin, SpyPlugin},
    prelude::{DmxAddress, Fixture, FixtureDefId, UniverseId},
};

#[test]
fn engine_can_start_function() {
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

    let fun = Effect::new_simple("Scene 1");
    // TODO: Add steps here
    let fun_id = fun.id();
    doc.add_effect(fun).unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (message_tx, _message_rx) = mpsc::channel();
    let engine = Engine::new(doc.state_view(), command_rx, message_tx);
    let handle = std::thread::spawn(|| {
        engine.start_loop();
    });

    let spy_plugin = Box::new(SpyPlugin::new());
    let p_id = spy_plugin.id();
    let data = Arc::clone(&spy_plugin.data);
    command_tx
        .send(EngineCommand::UniverseAdded(UniverseId::new(0)))
        .unwrap();
    command_tx
        .send(EngineCommand::AddPlugin(spy_plugin))
        .unwrap();
    command_tx
        .send(EngineCommand::AddPluginDestination {
            plugin: p_id,
            dest_universe: UniverseId::new(0),
        })
        .unwrap();
    command_tx
        .send(EngineCommand::StartFunction(fun_id))
        .unwrap();

    std::thread::sleep(Duration::from_secs(2));

    command_tx.send(EngineCommand::StopFunction).unwrap();

    command_tx.send(EngineCommand::Shutdown).unwrap();

    std::thread::sleep(Duration::from_millis(500));

    let guard = data.try_read().unwrap();
    let frame = guard[0].as_slice();
    assert_eq!(frame[0], 255);
    assert_eq!(frame[1], 200);
    handle.join().unwrap();
}
