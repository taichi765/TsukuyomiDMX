//! [`Doc`]に登録したコールバックが呼ばれたとき、[`FixtureMapModel`]のlistenerに通知が行く。
//! listenerは[`FixtureMapModel::row_data()`]を呼ぶことで最新のデータを得る。

use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashMap, pin::Pin, rc::Rc};

use i_slint_core::model::ModelChangeListener;
use slint::{Model, ModelNotify, ModelTracker};
use tsukuyomi_core::{
    doc::{Doc, DocEffect, DocStateView},
    prelude::{Fixture, FixtureDef, FixtureDefId, FixtureId},
};

/// [`Fixture`]をマップする`Model`
#[derive(Debug)]
pub struct FixtureMapModel<F> {
    f: F,
    inner: Rc<FixtureModelInner>,
}

impl<F, R> FixtureMapModel<F>
where
    F: Fn(&Fixture) -> R,
{
    pub fn new(inner: Rc<FixtureModelInner>, f: F) -> Self {
        Self { f, inner }
    }
}

impl<F, R> Model for FixtureMapModel<F>
where
    F: 'static + Fn(&Fixture) -> R,
{
    type Data = R;

    fn row_count(&self) -> usize {
        self.inner.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.inner.with_row(row, |fxt| (self.f)(fxt))
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.inner.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 複数の[`FixtureMapModel`]の間で共有される。
#[derive(derive_more::Debug)]
pub struct FixtureModelInner {
    #[debug(skip)]
    state: DocStateView,
    #[debug(skip)]
    notify: ModelNotify,
    keys: RefCell<Vec<FixtureId>>,
    index: RefCell<HashMap<FixtureId, usize>>,
}

impl FixtureModelInner {
    fn create(doc: &mut Doc) -> Rc<Self> {
        let me = Rc::new(Self {
            state: doc.state_view(),
            notify: ModelNotify::default(),
            keys: RefCell::new(Vec::new()),
            index: RefCell::new(HashMap::new()),
        });

        let me_clone = Rc::clone(&me);
        doc.subscribe(Box::new(move |ef| match ef {
            DocEffect::FixtureAdded(id) => {
                me.keys.borrow_mut().push(*id);
                me.index
                    .borrow_mut()
                    .insert(*id, me.keys.borrow().len() - 1);
                me.notify
                    .row_added(me.index.borrow().get(id).expect("todo").to_owned(), 1);
            }
            DocEffect::FixtureUpdated(id) => {
                let row = me.index.borrow().get(id).expect("todo").to_owned();

                me.notify.row_changed(row)
            }
            DocEffect::FixtureRemoved(id) => {
                let row = me.index.borrow().get(id).expect("todo").to_owned();
                me.keys.borrow_mut().remove(row);
                me.index.borrow_mut().remove(id);
                me.index.borrow_mut().iter_mut().for_each(|(_, r)| {
                    if *r >= row {
                        *r -= 1
                    }
                });
                me.notify.row_removed(row, 1)
            }
            _ => (),
        }));
        me_clone
    }

    fn row_count(&self) -> usize {
        self.state.with_fixtures(|it| it.iter().count())
    }

    fn with_row<F, R>(&self, idx: usize, f: F) -> Option<R>
    where
        F: FnOnce(&Fixture) -> R,
    {
        let id = self.keys.borrow().get(idx).expect("todo").to_owned();
        self.state.with_fixtures(|it| {
            let fxt = it.get(&id)?;
            Some(f(fxt))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use i_slint_core::model::ModelChangeListenerContainer;
    use slint::{SharedString, ToSharedString};
    use tsukuyomi_core::doc::FakeFixtureDefRegistry;
    use tsukuyomi_core::doc::{FixtureDefLookupError, FixtureDefMetaData, FixtureDefRegistry};
    use tsukuyomi_core::fixture::FixtureChange;
    use tsukuyomi_core::prelude::*;

    use crate::models::test_helpers::{DummyModelChangeEvent, SpyModelPeer, create_fixture_def};

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct DummyStruct {
        name: SharedString,
        id: SharedString,
    }

    #[test]
    fn fixture_map_model_works() {
        let mut def_rg = FakeFixtureDefRegistry::new();

        let def = create_fixture_def();
        let def_id = def.id().to_owned();
        def_rg.insert(def_id.clone(), def);

        let mut doc = Doc::new_with_def_registry(Box::new(def_rg));
        let inner = FixtureModelInner::create(&mut doc);
        let map_model = FixtureMapModel::new(Rc::clone(&inner), |fxt| DummyStruct {
            name: fxt.name().to_shared_string(),
            id: fxt.id().to_shared_string(),
        });

        let container = Box::pin(ModelChangeListenerContainer::new(SpyModelPeer::new()));
        map_model
            .model_tracker()
            .attach_peer(container.as_ref().model_peer());

        let name = "Test Fixture";

        let fxt = Fixture::new(
            name.to_string(),
            UniverseId::new(0),
            DmxAddress::new(1).unwrap(),
            def_id,
            "4 Channel",
            0.,
            0.,
        );
        let fxt_id = fxt.id();

        doc.add_fixture(fxt).unwrap();

        assert_eq!(1, inner.row_count());
        assert_eq!(fxt_id, inner.keys.borrow()[0]);
        assert_eq!(0, *inner.index.borrow().get(&fxt_id).unwrap());

        assert_eq!(
            DummyStruct {
                name: name.to_shared_string(),
                id: fxt_id.to_shared_string()
            },
            map_model.row_data(0).unwrap()
        );
        matches!(
            container.events.borrow()[0],
            DummyModelChangeEvent::Added(0)
        );

        let new_name = "Renamed Fixture";
        doc.update_fixture(fxt_id, FixtureChange::Rename(new_name.to_string()))
            .unwrap();
        assert_eq!(2, container.events.borrow().len());
        assert_eq!(
            DummyStruct {
                name: new_name.to_shared_string(),
                id: fxt_id.to_shared_string()
            },
            map_model.row_data(0).unwrap()
        );
        matches!(
            container.events.borrow()[1],
            DummyModelChangeEvent::Changed(0)
        );

        doc.remove_fixture(&fxt_id).unwrap();
        assert_eq!(0, inner.row_count());
        assert_eq!(0, inner.index.borrow().len());
        assert_eq!(0, inner.keys.borrow().len());
        assert_eq!(3, container.events.borrow().len());
        matches!(
            container.events.borrow()[2],
            DummyModelChangeEvent::Removed(0)
        );
    }
}
