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
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak, mpsc};
use std::time::Duration;

use i_slint_backend_winit::WinitWindowAccessor;

use slint::wgpu_28::{WGPUConfiguration, WGPUSettings};
use slint::{Timer, TimerMode};
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt, prelude::*};
use tsukuyomidmx_core::engine::{Engine, EngineCommand, EngineMessage};
use tsukuyomidmx_core::prelude::*;

use crate::app::App;

mod ui {
    slint::include_modules!();
}

pub fn run_main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

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
