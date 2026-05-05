use std::{
    rc::Rc,
    sync::mpsc::{self, Sender},
    time::Duration,
};

use anyhow::Context;
use slint::{ComponentHandle, Timer, TimerMode};
use tokio::sync::watch;
use tracing::instrument;
use tsukuyomidmx_core::{
    engine::EngineCommand,
    plugins::{DmxFrame, OutputPluginId, Plugin, PluginMessage},
    prelude::UniverseId,
};

use crate::{app::App, models::Preview2DModel, ui};

#[instrument(skip_all)]
pub fn setup(app: &App) {
    let adopter = app.ui.global::<ui::Preview2DAdopter>();

    let (frame_tx, frame_rx) = mpsc::channel();

    let model = Rc::new(Preview2DModel::new(
        Rc::clone(&app.shared_model.fixture_model.get().unwrap()),
        app.doc.lock().unwrap().state_view(),
        frame_rx,
    ));

    // UniverseごとにAddPluginDestinationする
    let universes = app.doc.lock().unwrap().state_view().universes();
    universes.into_iter().for_each(|u| {
        app.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::AddPlugin(Box::new(Preview2DPlugin::new(
                u,
                frame_tx.clone(),
            ))))
            .unwrap()
    });

    adopter.set_model(Rc::clone(&model).into());

    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        model.update();
    });
    if let Err(_) = app.preview2d_timer.set(timer) {
        unreachable!()
    };
}

/// Just sends message to [PreviewController] when [`Engine`][tsukuyomi_core::engine::Engine] ticked
#[derive(derive_more::Debug)]
struct Preview2DPlugin {
    id: OutputPluginId,
    universe: UniverseId,
    #[debug(skip)]
    frame_tx: Sender<(UniverseId, DmxFrame)>,
}

impl Preview2DPlugin {
    pub fn new(universe: UniverseId, frame_tx: Sender<(UniverseId, DmxFrame)>) -> Self {
        Self {
            id: OutputPluginId::new(),
            universe,
            frame_tx,
        }
    }
}

impl Plugin for Preview2DPlugin {
    fn id(&self) -> OutputPluginId {
        self.id
    }

    fn universe(&self) -> UniverseId {
        self.universe
    }

    fn start(
        self: Box<Self>,
        mut rx: watch::Receiver<PluginMessage>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), anyhow::Error>> + Send>> {
        Box::pin(async move {
            loop {
                rx.changed()
                    .await
                    .expect("it seems that engine has been panicked"); // TODO: engineをrestartのほうが良い？
                match &*rx.borrow_and_update() {
                    PluginMessage::DmxFrame(frame) => {
                        self.frame_tx.send((self.universe, frame.clone())).unwrap();
                    }
                    PluginMessage::Stop => break,
                };
            }
            anyhow::Ok(())
        })
    }
}

#[cfg(test)]
mod tests {}
