use std::{
    rc::Rc,
    sync::mpsc::{self, Sender},
};

use slint::ComponentHandle;
use tracing::instrument;
use tsukuyomidmx_core::{
    doc::OutputPluginId,
    engine::EngineCommand,
    plugins::{DmxFrame, Plugin},
    prelude::UniverseId,
};

use crate::{app::App, models::Preview2DModel, ui};

#[instrument(skip_all)]
pub fn setup(app: &mut App) {
    let adopter = app.ui.global::<ui::Preview2DAdopter>();

    let (frame_tx, frame_rx) = mpsc::channel();

    let model = Rc::new(Preview2DModel::new(
        Rc::clone(&app.shared_model_inner.fixture_model.get().unwrap()),
        app.doc.lock().unwrap().state_view(),
        frame_rx,
    ));

    let plugin = Box::new(Preview2DPlugin::new(frame_tx));
    app.command_tx
        .get()
        .unwrap()
        .send(EngineCommand::AddPlugin(plugin))
        .unwrap();
    // TODO: UniverseごとにAddPluginDestinationする

    adopter.set_fixture_list(Rc::clone(&model).into());
    todo!()
}

/// Just sends message to [PreviewController] when [`Engine`][tsukuyomi_core::engine::Engine] ticked
#[derive(derive_more::Debug)]
struct Preview2DPlugin {
    id: OutputPluginId,
    #[debug(skip)]
    frame_tx: Sender<(UniverseId, DmxFrame)>,
}

impl Preview2DPlugin {
    pub fn new(frame_tx: Sender<(UniverseId, DmxFrame)>) -> Self {
        Self {
            id: OutputPluginId::new(),
            frame_tx,
        }
    }
}

impl Plugin for Preview2DPlugin {
    fn send_dmx(&self, universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error> {
        self.frame_tx
            .send((universe_id, dmx_data))
            .expect("failed to send message from preview plugin to preview model");
        Ok(())
    }

    fn id(&self) -> OutputPluginId {
        self.id
    }
}

#[cfg(test)]
mod tests {}
