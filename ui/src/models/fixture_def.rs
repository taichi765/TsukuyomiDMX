use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use slint::{ModelNotify, SharedString};
use tsukuyomi_core::{
    doc::{Doc, DocEffect, DocStateView},
    prelude::{FixtureDef, FixtureDefId},
};

pub struct FixtureDefMapModel<F> {
    f: F,
    inner: Rc<FixtureDefModelInner>,
}

impl<F, R> FixtureDefMapModel<F>
where
    F: Fn(&FixtureDef) -> R,
{
    pub fn new(inner: Rc<FixtureDefModelInner>, f: F) -> Self {
        Self { f, inner }
    }
}

pub struct FixtureDefModelInner {
    state: DocStateView,
    catalog: RefCell<HashMap<SharedString, Vec<(FixtureDefId, SharedString)>>>,
    notify: ModelNotify,
}

impl FixtureDefModelInner {
    pub fn create(doc: &mut Doc) -> Rc<Self> {
        let me = Rc::new(Self {
            state: doc.state_view(),
            catalog: RefCell::new(HashMap::new()),
            notify: ModelNotify::default(),
        });
        let me_clone = Rc::clone(&me);
        doc.subscribe(Box::new(move |ef| match ef {
            DocEffect::DefRegistryLoaded => {
                let new = me.state.with_fixture_defs(|it| {
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
                *me.catalog.borrow_mut() = new;
                me.notify.reset();
            }
            _ => (),
        }));
        me_clone
    }

    pub fn with_row<F, R>(&self, row: usize, f: F) -> R
    where
        F: FnOnce(&FixtureDef) -> R,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsukuyomi_core::doc::{Doc, FakeFixtureDefRegistry};

    #[test]
    fn def_map_model_works() {
        let doc = Doc::new_with_def_registry(Box::new(FakeFixtureDefRegistry::new()));
    }
}
