#![allow(unused_imports)]

pub mod app;
pub mod colors;
//pub mod controllers;
pub mod models;
pub mod tea {
    pub mod fixture_list_view;
    pub mod universe_view;
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
use tracing_subscriber::fmt::format::FmtSpan;
use tsukuyomi_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomi_core::prelude::*;

use crate::app::App;

mod ui {
    slint::include_modules!();
}

pub fn run_main() -> Result<(), Box<dyn Error>> {
    // TODO: TSUKUYOMIDMX_LOG_LEVELで指定できるようにする
    // Initialize logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
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
