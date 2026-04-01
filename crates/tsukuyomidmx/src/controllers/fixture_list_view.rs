use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use slint::{ComponentHandle, Model, ToSharedString, VecModel, Weak};
use tracing::debug;
use tsukuyomi_core::prelude::*;
use tsukuyomi_core::{
    ReadOnly,
    command_manager::CommandManager,
    commands::doc_commands,
    doc::{DocEvent, DocEventBus, DocObserver},
};
use uuid::Uuid;

use crate::{AppWindow, FixtureListStore, FixtureModel, ManufacturerModel};

pub fn setup_fixture_list_view(
    ui: &AppWindow,
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
    command_manager: Rc<RefCell<CommandManager>>,
) -> Arc<RwLock<FixtureListViewController>> {
    let controller = Arc::new(RwLock::new(FixtureListViewController::new(
        ui.as_weak(),
        ReadOnly::clone(&doc),
    )));
    event_bus.subscribe(Arc::downgrade(&controller) as _);

    let doc_clone = ReadOnly::clone(&doc);
    ui.global::<FixtureListStore>()
        .on_get_next_address(move |universe| {
            let universe_id = parse_universe_id(universe.as_str());
            doc_clone
                .read()
                .current_max_address(universe_id)
                .map_or(0, |adr| adr.value() as i32 + 1)
        });

    // Reset dummy model
    ui.global::<FixtureListStore>()
        .set_model(Rc::new(VecModel::from(Vec::new())).into());
    controller
}

pub struct FixtureListViewController {
    ui_handle: Weak<AppWindow>,
    doc: ReadOnly<DocStore>,
}

impl FixtureListViewController {
    fn new(ui_handle: Weak<AppWindow>, doc: ReadOnly<DocStore>) -> Self {
        Self { ui_handle, doc }
    }
}

impl DocObserver for FixtureListViewController {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::FixtureDefAdded(def_id) => {
                let ui = self.ui_handle.unwrap();
                let store = ui.global::<FixtureListStore>();
                let doc = self.doc.read();

                let def = doc.get_fixture_def(def_id).unwrap();
                let def_id = def_id.to_shared_string();

                let new_manufacturer = def.manufacturer();
                let new_fixture_model = FixtureModel {
                    id: def_id.to_shared_string(),
                    name: def.model().to_shared_string(),
                    modes: Rc::new(VecModel::from(
                        def.modes()
                            .keys()
                            .map(|s| s.to_shared_string())
                            .collect::<Vec<_>>(),
                    ))
                    .into(),
                };

                let mut manufacturers: Vec<ManufacturerModel> = store.get_model().iter().collect();
                if let Some(m) = manufacturers
                    .iter()
                    .find(|m| m.manufacturer == new_manufacturer)
                {
                    let existing_idx = m.fixtures.iter().position(|fxt| fxt.id == def_id);
                    let idx = existing_idx.unwrap_or_else(|| m.fixtures.iter().count());
                    m.fixtures.set_row_data(idx, new_fixture_model);
                } else {
                    manufacturers.push(ManufacturerModel {
                        expanded: false,
                        fixtures: Rc::new(VecModel::from(vec![new_fixture_model])).into(),
                        manufacturer: new_manufacturer.to_shared_string(),
                    });
                }

                store.set_model(Rc::new(VecModel::from(manufacturers)).into());
            }
            DocEvent::FixtureDefUpdated(id) => todo!(),
            DocEvent::FixtureDefRemoved(id) => todo!(),
            _ => (),
        }
    }
}

impl Drop for FixtureListViewController {
    fn drop(&mut self) {
        debug!("FixtureListViewController is dropping");
    }
}
