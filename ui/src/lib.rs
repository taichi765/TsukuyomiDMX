#![allow(unused_imports)]

pub mod app;
pub mod colors;
//pub mod controllers;
pub mod models;
pub mod tea {
    pub mod fixture_list_view;
}
mod test_helpers;

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak, mpsc};
use std::time::Duration;

use i_slint_backend_winit::WinitWindowAccessor;

use slint::wgpu_28::{WGPUConfiguration, WGPUSettings};
use slint::{Timer, TimerMode};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::prelude::*;

use crate::app::App;

mod ui {
    slint::include_modules!();
}

pub fn run_main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Use wgpu to render 3D Preview
    slint::BackendSelector::new()
        .require_wgpu_28(WGPUConfiguration::Automatic(WGPUSettings::default()))
        .select()
        .expect("unable to create Slint backend WGPU based renderer");

    // TODO: language switch(preferences)
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/translations/"));

    let mut app = App::new();
    app.run()?;
    Ok(())
}

/*fn hoge() {
    // HACK: depending on the order of initialization, it would cause crash.
    let doc = Arc::new(RwLock::new(DocState::new()));
    let mut event_bus = DocEventBus::new();

    // HACK: Initialize all observers before changing Doc.
    // If Doc is changed before engine initialized, states in observers(and engine) would be invalid.
    let (engine_handle, command_tx, error_rx, _bridge) =
        setup_engine(ArcView::new(Arc::clone(&doc)), &mut event_bus);

    let ui = setup_window().expect("failed to setup ui");

    let mut doc_commands = Vec::new();

    for i in 1..5 {
        doc_commands.push(Box::new(doc_commands::AddUniverse::new(UniverseId::new(i))) as _);
    }

    let (mut dc, fixture_id) = create_some_presets();
    doc_commands.append(&mut dc);

    setup_fader_view(&ui, command_tx.clone(), fixture_id);
    let (mut dc, mut update_2d_preview) = setup_2d_preview(
        &ui,
        ArcView::new(Arc::clone(&doc)),
        &mut event_bus,
        command_tx.clone(),
    );
    doc_commands.append(&mut dc);
    setup_3d_preview(&ui);

    let event_bus = Rc::new(RefCell::new(event_bus));

    let command_manager = Rc::new(RefCell::new(CommandManager::new(DocHandle::new(
        Arc::clone(&doc),
        Rc::clone(&event_bus),
    ))));

    let _controller = setup_fixture_list_view(
        &ui,
        ArcView::new(Arc::clone(&doc)),
        &mut event_bus.borrow_mut(),
        Rc::clone(&command_manager),
    );
    let _controller = setup_universe_view(
        &ui,
        ArcView::new(Arc::clone(&doc)),
        &mut event_bus.borrow_mut(),
    );

    doc_commands.into_iter().for_each(|cmd| {
        command_manager.borrow_mut().execute(cmd).unwrap();
    });

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(33), move || {
        update_2d_preview();
    });

    ui.run()?;

    if let Err(e) = command_tx.send(EngineCommand::Shutdown) {
        eprintln!("failed to send message to engine:{}", e);
    }

    engine_handle.join().unwrap();
}*/

/*fn setup_engine(
    doc: ArcView<DocState>,
    event_bus: &mut DocEventBus,
) -> (
    std::thread::JoinHandle<()>,
    Sender<EngineCommand>,
    Receiver<EngineMessage>,
    Arc<RwLock<DocEventBridge>>,
) {
    let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
    let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

    let bridge = Arc::new(RwLock::new(DocEventBridge::new(command_tx.clone())));
    let bridge_weak: Weak<RwLock<dyn DocObserver>> = Arc::downgrade(&bridge) as _;
    event_bus.subscribe(bridge_weak);

    let engine = Engine::new(doc, command_rx, error_tx);

    let engine_handle = std::thread::Builder::new()
        .name("tsukuyomi-engine".into())
        .spawn(move || engine.start_loop())
        .unwrap();
    (engine_handle, command_tx, error_rx, bridge)
}*/

/*fn setup_window() -> Result<AppWindow, Box<dyn Error>> {
    slint::BackendSelector::new()
        .require_wgpu_27(WGPUConfiguration::Automatic(WGPUSettings::default()))
        .select()
        .expect("unable to create Slint backend WGPU based renderer");

    let ui = AppWindow::new()?;
    // TODO: language switch(preferences)
    slint::select_bundled_translation("en".into()).unwrap();

    let ui_handle = ui.as_weak();
    ui.on_start_drag(move || {
        let ui = ui_handle.unwrap();
        ui.window().with_winit_window(|w| w.drag_window());
    });

    let ui_handle = ui.as_weak();
    ui.on_minimize(move || {
        let ui = ui_handle.unwrap();
        ui.window().set_minimized(true);
    });

    // TODO: toggle-fullscreen

    let ui_handle = ui.as_weak();
    ui.on_close(move || {
        let ui = ui_handle.unwrap();
        ui.window().hide().unwrap()
    });

    Ok(ui)
}*/

/*fn create_some_presets() -> (Vec<Box<dyn DocCommand>>, FixtureId) {
    let mut commands: Vec<Box<dyn DocCommand>> = Vec::new();

    let fixture_def_id = {
        let mut fixture_def = FixtureDef::new("Test Manufacturer", "Test Model");
        fixture_def.insert_channel(
            "Dimmer",
            ChannelDef::new(MergeMode::HTP, ChannelKind::Dimmer),
        );
        fixture_def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, ChannelKind::Red));
        let channel_order = vec![("Dimmer".to_string(), 0usize), ("Red".to_string(), 1)];
        let mode = FixtureMode::new(channel_order.into_iter()).unwrap();
        fixture_def.insert_mode("Mode 1", mode);

        let id = fixture_def.id();

        commands.push(Box::new(commands::doc_commands::AddFixtureDef::new(
            fixture_def,
        ))); // FIXME: 式の中で副作用があるのは良くない気がする
        id
    };

    let fixture = Fixture::new(
        "Test Fixture",
        UniverseId::new(1),
        DmxAddress::new(1).unwrap(),
        fixture_def_id,
        "Mode 1".into(),
        200.,
        200.,
    );

    let fixture_id = fixture.id();
    commands.push(Box::new(commands::doc_commands::AddFixture::new(fixture)));

    {
        let mut fixture_def = FixtureDef::new("Stage Evolution", "HPAR64-9");
        {
            fixture_def.insert_channel(
                "Dimmer",
                ChannelDef::new(MergeMode::HTP, ChannelKind::Dimmer),
            );
            fixture_def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, ChannelKind::Red));
            fixture_def
                .insert_channel("Green", ChannelDef::new(MergeMode::HTP, ChannelKind::Green));
            fixture_def.insert_channel("Blue", ChannelDef::new(MergeMode::HTP, ChannelKind::Blue));
            fixture_def
                .insert_channel("White", ChannelDef::new(MergeMode::HTP, ChannelKind::White));
            fixture_def.insert_channel(
                "Warm White",
                ChannelDef::new(MergeMode::HTP, ChannelKind::WarmWhite),
            );
            fixture_def
                .insert_channel("Amber", ChannelDef::new(MergeMode::HTP, ChannelKind::Amber));
            fixture_def.insert_channel("UV", ChannelDef::new(MergeMode::HTP, ChannelKind::UV));
            fixture_def.insert_channel(
                "Color Macro",
                ChannelDef::new(MergeMode::HTP, ChannelKind::Custom),
            );
            fixture_def.insert_channel(
                "Auto Program",
                ChannelDef::new(MergeMode::HTP, ChannelKind::Custom),
            );
            fixture_def.insert_channel(
                "Program Speed",
                ChannelDef::new(MergeMode::HTP, ChannelKind::Custom),
            );
            fixture_def.insert_channel(
                "Color Temprature",
                ChannelDef::new(MergeMode::HTP, ChannelKind::Custom),
            );
        }
        let channel_order = vec![
            ("Dimmer", 0usize),
            ("Red", 1),
            ("Green", 2),
            ("Blue", 3),
            ("White", 4),
            ("Warm White", 5),
            ("Amber", 6),
            ("UV", 7),
            ("Color Macro", 8),
            ("Auto Program", 9),
            ("Program Speed", 10),
            ("Color Temprature", 11),
        ];
        let channel_order = channel_order
            .into_iter()
            .map(|(ch, off)| (ch.to_string(), off));
        let mode = FixtureMode::new(channel_order).unwrap();
        fixture_def.insert_mode("13ch", mode);

        commands.push(Box::new(doc_commands::AddFixtureDef::new(fixture_def)));
    }

    (commands, fixture_id)
}*/
