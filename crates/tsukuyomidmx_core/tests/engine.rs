use std::{
    sync::{Arc, RwLock, mpsc},
    time::Duration,
};

use tsukuyomidmx_core::{
    doc::{Doc, OutputPluginId},
    engine::{Engine, EngineCommand},
    functions::SimpleFunction,
    plugins::{DmxFrame, Plugin},
    prelude::{DmxAddress, Fixture, FixtureDefId, UniverseId},
};

struct SpyPlugin {
    id: OutputPluginId,
    data: Arc<RwLock<Vec<DmxFrame>>>,
}

impl Plugin for SpyPlugin {
    fn id(&self) -> tsukuyomidmx_core::prelude::OutputPluginId {
        self.id
    }

    fn send_dmx(&self, _universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error> {
        self.data.write().unwrap().push(dmx_data);
        println!("sending...");
        Ok(())
    }
}

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
    let fxt_id = fxt.id();
    doc.add_fixture(fxt).unwrap();

    let fun = SimpleFunction::new(
        vec![((fxt_id, 0), 255), ((fxt_id, 1), 255)]
            .into_iter()
            .collect(),
    );
    let fun_id = fun.id();
    doc.add_function(fun).unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (message_tx, _message_rx) = mpsc::channel();
    let engine = Engine::new(doc.state_view(), command_rx, message_tx);
    let handle = std::thread::spawn(|| {
        engine.start_loop();
    });
    let spy_plugin = SpyPlugin {
        id: OutputPluginId::new(),
        data: Arc::new(RwLock::new(Vec::new())),
    };
    let p_id = spy_plugin.id;
    doc.add_output_plugin(UniverseId::new(0), todo!()).unwrap();

    let data = Arc::clone(&spy_plugin.data);
    command_tx
        .send(EngineCommand::UniverseAdded(UniverseId::new(0)))
        .unwrap();
    command_tx
        .send(EngineCommand::AddPlugin(Box::new(spy_plugin)))
        .unwrap();
    command_tx
        .send(EngineCommand::StartFunction(fun_id))
        .unwrap();

    std::thread::sleep(Duration::from_secs(2));

    command_tx.send(EngineCommand::StopFunction).unwrap();

    command_tx.send(EngineCommand::Shutdown).unwrap();

    println!("{:?}", data.read().unwrap());

    handle.join().unwrap();
}
