use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use slint::{ComponentHandle, ToSharedString};
use tracing::debug;
use tsukuyomi_core::{
    ReadOnly,
    doc::{DocEvent, DocEventBus, DocObserver},
    prelude::*,
};

use crate::{AppWindow, UniverseViewFixtureModel, UniverseViewStore, hashmap_model::HashMapModel};

pub fn setup_universe_view(
    ui: &AppWindow,
    doc: ReadOnly<DocStore>,
    event_bus: &mut DocEventBus,
) -> Arc<RwLock<UniverseViewController>> {
    let controller = Arc::new(RwLock::new(UniverseViewController::new(doc)));
    event_bus.subscribe(Arc::downgrade(&controller) as _);

    let model_clone = Rc::clone(&controller.read().unwrap().model);
    ui.global::<UniverseViewStore>()
        .set_fixtures(Rc::new(model_clone).into());
    controller
}

pub struct UniverseViewController {
    doc: ReadOnly<DocStore>,
    model: Rc<HashMapModel<FixtureId, UniverseViewFixtureModel>>,
}

impl UniverseViewController {
    fn new(doc: ReadOnly<DocStore>) -> Self {
        Self {
            doc,
            model: Rc::new(HashMapModel::new()),
        }
    }
}

impl DocObserver for UniverseViewController {
    fn on_doc_event(&mut self, event: &DocEvent) {
        match event {
            DocEvent::FixtureAdded(id) => {
                let doc = self.doc.read();
                let fixture = doc.get_fixture(id).unwrap(); // FIXME: unwrap
                let def = doc.get_fixture_def(&fixture.fixture_def()).unwrap();
                self.model.insert(
                    *id,
                    UniverseViewFixtureModel {
                        address: fixture.address().value() as i32,
                        fixture_id: id.to_shared_string(),
                        name: fixture.name().to_shared_string(),
                        footprint: fixture.footprint(def).unwrap() as i32,
                    },
                );
            }
            DocEvent::FixtureUpdated(id) => {}
            DocEvent::FixtureRemoved(id) => {}
            _ => (),
        }
    }
}

impl Drop for UniverseViewController {
    fn drop(&mut self) {
        debug!("UniverseViewController is dropping");
    }
}
