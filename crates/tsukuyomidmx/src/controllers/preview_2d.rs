use std::{
    collections::HashMap,
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{self, Receiver, Sender},
    },
};

use slint::{Color, ComponentHandle, LogicalPosition, ToSharedString, Weak};
use tracing::debug;
use tsukuyomi_core::{
    ReadOnly,
    commands::{DocCommand, doc_commands},
    doc::{DocEvent, DocEventBus, DocObserver},
    engine::EngineCommand,
    fixture_def::ChannelKind,
    plugins::{DmxFrame, Plugin},
    prelude::*,
};

use crate::{
    AppWindow, EditorTabs, FixtureEntityData, Preview2DStore, TopLevelTabs, colors::ColorInfo,
    hashmap_model::HashMapModel,
};

/// Returns closure to update preview, which should be called in [slint::Timer]
pub fn setup_2d_preview(
    ui: &AppWindow,
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
    command_tx: Sender<EngineCommand>,
) -> (Vec<Box<dyn DocCommand>>, impl FnMut() -> () + 'static) {
    let (msg_tx, msg_rx) = mpsc::channel::<PreviewMessage>();
    let controller = Arc::new(RwLock::new(PreviewController::new(
        doc,
        ui.as_weak(),
        Mutex::new(msg_rx),
    )));

    event_bus.subscribe(Arc::downgrade(&controller) as _);
    let model_clone = Rc::clone(&controller.read().unwrap().model);
    ui.global::<Preview2DStore>()
        .set_fixture_list(model_clone.into());

    // TODO: シリアライズからの復元
    let plugin = PreviewPlugin::new(msg_tx);
    let p_id = plugin.id();

    command_tx
        .send(EngineCommand::AddPlugin(Box::new(plugin)))
        .expect("failed to send command to engine"); // FIXME: error handling

    let doc_commands: Vec<Box<dyn DocCommand>> = vec![Box::new(doc_commands::AddOutput::new(
        UniverseId::new(1), //FIXME: hard coding
        p_id,
    ))];

    (doc_commands, move || {
        controller.write().unwrap().update();
    })
}

/// Just sends message to [PreviewController] when [`Engine`][tsukuyomi_core::engine::Engine] ticked
struct PreviewPlugin {
    id: OutputPluginId,
    msg_tx: Sender<PreviewMessage>,
}

impl PreviewPlugin {
    pub fn new(msg_tx: Sender<PreviewMessage>) -> Self {
        Self {
            id: OutputPluginId::new(),
            msg_tx,
        }
    }
}

impl Plugin for PreviewPlugin {
    fn send_dmx(&self, universe_id: UniverseId, dmx_data: DmxFrame) -> Result<(), std::io::Error> {
        self.msg_tx
            .send(PreviewMessage::DmxFrame {
                universe_id,
                dmx_data,
            })
            .expect("failed to send message from preview plugin to preview controller");
        Ok(())
    }

    fn id(&self) -> OutputPluginId {
        self.id
    }
}

/// Controls actual Preview UI
struct PreviewController {
    doc: ReadOnly<DocStore>,
    ui_handle: Weak<AppWindow>,
    model: Rc<HashMapModel<FixtureId, FixtureEntityData>>,
    msg_rx: Mutex<Receiver<PreviewMessage>>,
}

impl PreviewController {
    fn new(
        doc: ReadOnly<DocStore>,
        ui_handle: Weak<AppWindow>,
        msg_rx: Mutex<Receiver<PreviewMessage>>,
    ) -> Self {
        Self {
            doc,
            ui_handle,
            model: Rc::new(HashMapModel::new()),
            msg_rx,
        }
    }

    fn update_fixture_model(&mut self, id: FixtureId) {
        let doc = self.doc.read();
        let fixture = doc.get_fixture(&id).unwrap();
        self.model.insert(
            id,
            FixtureEntityData {
                color: Color::default(),
                fixture_id: id.to_shared_string(),
                pos: LogicalPosition {
                    x: fixture.x(),
                    y: fixture.y(),
                },
            },
        );
    }

    /// Update preview based on the message received from [`PreviewPlugin`].
    fn update(&mut self) {
        let ui = self.ui_handle.unwrap();
        if ui.get_current_tab() != TopLevelTabs::Editor {
            return;
        }
        if ui.get_editor_tab_current_index() != EditorTabs::Preview2D {
            return; // FIXME: これだけのためにui_handleを持っておくのってどうなんだろう...
        }
        // FIXME: unwrap
        while let Ok(msg) = self.msg_rx.lock().unwrap().try_recv() {
            match msg {
                PreviewMessage::DmxFrame {
                    universe_id,
                    dmx_data,
                } => self.apply_dmx_frame(universe_id, dmx_data),
            }
        }
    }

    /// Actually applies dmx frame to UI's model.
    fn apply_dmx_frame(&self, universe_id: UniverseId, dmx_data: DmxFrame) {
        let mut fixture_color_map: HashMap<FixtureId, ColorInfo> = HashMap::new();

        let doc = self.doc.read();
        for (address, value) in dmx_data.iter() {
            let Some(&(fixture_id, offset)) = doc.get_fixture_by_address(&universe_id, address)
            //FIXME: キャッシュ
            else {
                continue;
            };

            let channel = {
                let fixture = doc.get_fixture(&fixture_id).unwrap();
                let def = doc.get_fixture_def(&fixture.fixture_def()).unwrap();
                let mode = def.modes().get(fixture.fixture_mode()).unwrap();
                let channel_name = mode.get_channel_by_offset(offset).unwrap();
                def.channel_templates().get(channel_name).unwrap()
            };

            set_color(fixture_id, &mut fixture_color_map, channel.kind(), value);
        }

        fixture_color_map.into_iter().for_each(|(fxt_id, color)| {
            let old = self.model.get(&fxt_id).unwrap();
            self.model.insert(
                fxt_id,
                FixtureEntityData {
                    color: color.to_slint_color(),
                    fixture_id: fxt_id.to_shared_string(),
                    ..old
                },
            ); // OPTIM: すべてのフィクスチャを更新してからnotifyの方が早いかも？
        });
    }
}

impl DocObserver for PreviewController {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::FixtureAdded(id) => self.update_fixture_model(*id),
            DocEvent::FixtureUpdated(id) => self.update_fixture_model(*id),
            DocEvent::FixtureRemoved(id) => {
                self.model.remove(id);
            }
            _ => (),
        }
    }
}

impl Drop for PreviewController {
    fn drop(&mut self) {
        debug!("PreviewController is dropping");
    }
}

enum PreviewMessage {
    DmxFrame {
        universe_id: UniverseId,
        dmx_data: DmxFrame,
    },
}

/// Helper function to set color based on `ChannelKind`.
fn set_color(
    fixture_id: FixtureId,
    map: &mut HashMap<FixtureId, ColorInfo>,
    kind: &ChannelKind,
    value: u8,
) {
    let mut color = map
        .get(&fixture_id)
        .copied()
        .or(Some(ColorInfo::default()))
        .unwrap();
    match kind {
        ChannelKind::Dimmer => color.dimmer = value,
        ChannelKind::Red => color.red = value,
        ChannelKind::Green => color.green = value,
        ChannelKind::Blue => color.blue = value,
        ChannelKind::White => color.white = value,
        ChannelKind::Amber => color.amber = value,
        ChannelKind::UV => color.uv = value,
        _ => (),
    }
    map.insert(fixture_id, color);
}
