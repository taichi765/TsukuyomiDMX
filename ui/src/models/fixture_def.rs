use core::any::Any;
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use slint::{Model, ModelNotify, ModelTracker, SharedString};
use tsukuyomi_core::{
    doc::{Doc, DocEffect, DocStateView},
    prelude::{FixtureDef, FixtureDefId},
};

type Manufacturer = SharedString;
type ModelName = SharedString;

#[derive(derive_more::Debug)]
pub struct FixtureDefModel {
    #[debug(skip)]
    state: DocStateView,
    catalog: RefCell<HashMap<Manufacturer, Vec<(FixtureDefId, ModelName)>>>,
    keys: RefCell<Vec<Manufacturer>>,
    #[debug(skip)]
    notify: ModelNotify,
}

impl FixtureDefModel {
    pub fn create(doc: &mut Doc) -> Rc<Self> {
        let me = Rc::new(Self {
            state: doc.state_view(),
            catalog: RefCell::new(HashMap::new()),
            keys: RefCell::new(Vec::new()),
            notify: ModelNotify::default(),
        });
        let me_clone = Rc::clone(&me);
        doc.subscribe(Box::new(move |ef| match ef {
            DocEffect::DefRegistryLoaded => me.update_catalog(),
            _ => (),
        }));
        me_clone.update_catalog();
        me_clone
    }

    fn update_catalog(&self) {
        let new = self.state.with_fixture_defs(|it| {
            it.iter_metadata().fold(HashMap::new(), |mut map, v| {
                if !map.contains_key(v.manufacturer) {
                    map.insert(v.manufacturer.into(), vec![(v.id.clone(), v.model.into())]);
                } else {
                    map.get_mut(v.manufacturer)
                        .unwrap()
                        .push((v.id.clone(), v.model.into()));
                };
                map
            })
        });
        *self.keys.borrow_mut() = new.keys().map(|k: &SharedString| k.to_owned()).collect();
        *self.catalog.borrow_mut() = new;
        self.notify.reset();
    }
}

impl Model for FixtureDefModel {
    type Data = (Manufacturer, Vec<(FixtureDefId, ModelName)>);

    fn row_count(&self) -> usize {
        self.keys.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let manufacturer = self.keys.borrow().get(row)?.to_owned();
        let val = self.catalog.borrow().get(&manufacturer)?.to_owned();
        Some((manufacturer, val)) // OPTIM: SharedStringはともかくVecのcloneは重い
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::models::test_helpers::{
        DummyModelChangeEvent, SpyModelPeer, create_fixture_def, create_fixture_def_2,
    };

    use super::*;
    use i_slint_core::model::ModelChangeListenerContainer;
    use tsukuyomi_core::doc::{Doc, FakeFixtureDefRegistry};

    #[test]
    fn def_map_model_works() {
        // Arrange
        let mut def_rg = FakeFixtureDefRegistry::new();
        let def = create_fixture_def();
        let def_id = def.id().to_owned();
        def_rg.insert(def_id.clone(), def);
        let def_2 = create_fixture_def_2();
        let def_id_2 = def_2.id().to_owned();
        def_rg.insert(def_id_2.clone(), def_2);

        let mut doc = Doc::new_with_def_registry(Box::new(def_rg));
        // Act
        let model = FixtureDefModel::create(&mut doc);

        // Assert
        assert_eq!(2, model.catalog.borrow().iter().count());
        assert_eq!(2, model.keys.borrow().len());

        // Arrange
        let container = Box::pin(ModelChangeListenerContainer::new(SpyModelPeer::new()));
        model
            .model_tracker()
            .attach_peer(container.as_ref().model_peer());

        // Act
        doc.reload_defs(); // TODO: Fakeでreloadのときにdefを追加するようにしたい

        // Assert
        matches!(container.events.borrow()[0], DummyModelChangeEvent::Reset);
        assert_eq!(2, model.catalog.borrow().iter().count());
        assert_eq!(2, model.keys.borrow().len());
    }
}
