#![allow(unused_imports)]

pub mod app;
pub mod colors;
//pub mod controllers;
pub mod models;
pub mod tea {
    pub mod fixture_list_view;
    pub mod preview_2d;
    pub mod universe_view;
}
mod test_helpers;

use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak, mpsc};
use std::time::Duration;

use i_slint_backend_winit::WinitWindowAccessor;

use slint::wgpu_28::{WGPUConfiguration, WGPUSettings};
use slint::{Timer, TimerMode};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt, prelude::*};
use tsukuyomidmx_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomidmx_core::prelude::*;

use crate::app::App;

mod ui {
    slint::include_modules!();
}

pub fn run_main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    let filter = if std::env::var("TSUKUYOMI_LOG").is_ok() {
        EnvFilter::try_from_env("TSUKUYOMI_LOG").expect("TSUKUYOMI_LOG's format was invalid. see https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html")
    } else {
        EnvFilter::try_new("tsukuyomidmx=debug,tsukuyomidmx-core=debug,off").unwrap()
    };
    let my_layer = fmt::layer()
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .with_filter(filter);
    let external_layer = fmt::layer().with_filter(EnvFilter::new(
        "tsukuyomidmx=off,tsukuyomidmx-core=off,info",
    ));

    tracing_subscriber::registry()
        .with(my_layer)
        .with(external_layer)
        .init();

    // Use wgpu to render 3D Preview
    slint::BackendSelector::new()
        .require_wgpu_28(WGPUConfiguration::Automatic(WGPUSettings::default()))
        .select()
        .expect("unable to create Slint backend WGPU based renderer");

    // TODO: language switch(preferences)
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/translations/"));

    let mut args = std::env::args();
    let app = if let Some(project_path) = args.nth(1) {
        Arc::new(App::from_dir(Path::new(&project_path))?)
    } else {
        Arc::new(App::new_empty())
    };

    app.run()?;
    Ok(())
}
