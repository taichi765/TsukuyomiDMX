use core::any::Any;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ops::ControlFlow,
    pin::Pin,
    rc::Rc,
};

use i_slint_core::model::{ModelChangeListener, ModelChangeListenerContainer};
use slint::{Model, ModelNotify, ModelTracker, SharedString, ToSharedString};
use tsukuyomidmx_core::{
    doc::DocStateView,
    prelude::{DmxAddress, Fixture, FixtureId, UniverseId},
};

use crate::{models::FixtureModel, ui};

/// FixtureModelをSlint側で扱いやすい形に変える
pub struct UniverseViewModel<S: SourceModel, P: FixtureInfoProvider>(
    Pin<Box<ModelChangeListenerContainer<ModelInner<S, P>>>>,
);

// TODO: FixtureModel経由じゃなくてDocに直でsubscribe()してもいい気がする
pub trait SourceModel {
    fn get_id(&self, idx: usize) -> Option<FixtureId>;
    /// 直近のremoveされたfixtureのidを返す。一度もremoveされていない場合はNone。
    fn get_removed_id(&self) -> Option<FixtureId>;
    fn model_tracker(&self) -> &dyn ModelTracker;
    fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId>;
}

impl SourceModel for FixtureModel {
    fn get_id(&self, idx: usize) -> Option<FixtureId> {
        self.with_row(idx, |fxt| fxt.id())
    }

    fn get_removed_id(&self) -> Option<FixtureId> {
        self.latest_removed()
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        self.model_tracker()
    }

    fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId> {
        self.with_fixture(fxt_id, |fxt| fxt.universe_id())
    }
}

/// [`FixtureInfo`]をcomputeするのに必要なプロパティたち
///
/// TestのときにDocStateViewをstubしたいのでトレイト化
pub trait FixtureInfoProvider {
    // TODO: occupied_addressesをもらったほうがいい
    fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId>;
    fn get_address(&self, fxt_id: FixtureId) -> Option<DmxAddress>;
    fn get_footprint(&self, fxt_id: FixtureId) -> Option<usize>;
    fn get_name(&self, fxt_id: FixtureId) -> Option<SharedString>;
}

impl FixtureInfoProvider for DocStateView {
    fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId> {
        self.with_fixtures(|it| {
            let fxt = it.get(&fxt_id)?;
            Some(fxt.universe_id())
        })
    }

    fn get_address(&self, fxt_id: FixtureId) -> Option<DmxAddress> {
        self.with_fixtures(|it| {
            let fxt = it.get(&fxt_id)?;
            Some(fxt.address())
        })
    }

    fn get_footprint(&self, fxt_id: FixtureId) -> Option<usize> {
        self.with_fixtures_and_defs(|fxts, defs| {
            let fxt = fxts.get(&fxt_id)?;
            let def = defs.get(fxt.fixture_def()).ok()?;
            Some(def.mode(fxt.fixture_mode()).unwrap().footprint())
        })
    }

    fn get_name(&self, fxt_id: FixtureId) -> Option<SharedString> {
        self.with_fixtures(|it| Some(it.get(&fxt_id)?.name().to_shared_string()))
    }
}

impl<S, P> Model for UniverseViewModel<S, P>
where
    S: SourceModel + 'static,
    P: FixtureInfoProvider + 'static,
{
    type Data = (UniverseId, ui::UniverseViewFixtureData);

    fn row_count(&self) -> usize {
        self.0
            .data
            .borrow()
            .values()
            .fold(0, |count, v| count + v.len())
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let guard = self.0.row_order.borrow();
        let (fxt_id, n) = guard.get(row)?;

        self.0
            .data
            .borrow()
            .get(fxt_id)?
            .iter()
            .nth(*n)
            .map(|info| {
                (
                    info.universe,
                    ui::UniverseViewFixtureData {
                        col: info.col as i32,
                        row: info.row as i32,
                        fixture_id: fxt_id.to_shared_string(),
                        is_odd: false, // default
                        length: info.length as i32,
                        text: info.text.clone(),
                    },
                )
            })
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.0.notify
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<S, P> UniverseViewModel<S, P>
where
    S: SourceModel + 'static,
    P: FixtureInfoProvider + 'static,
{
    pub fn new(source_model: Rc<S>, info_provider: P, col_count: usize) -> Self {
        let inner = Box::pin(ModelChangeListenerContainer::new(ModelInner::new(
            Rc::clone(&source_model),
            info_provider,
            col_count,
        )));

        source_model
            .model_tracker()
            .attach_peer(inner.as_ref().model_peer());

        Self(inner)
    }
}

struct ModelInner<S, P> {
    source_model: Rc<S>,
    info_provider: P,
    // OPTIM: ほとんどの場合FixtureInfoは一つだけなので、smallvec等使ってスタックに置いたほうがいい気がする
    // FIXME: Universeごとにわける
    /// 行マタギの場合一つのFixtureに対し[`FixtureInfo`]が複数存在する
    data: RefCell<HashMap<FixtureId, Vec<FixtureInfo>>>,
    row_order: RefCell<Vec<(FixtureId, usize)>>,
    /// Number of columns per row
    col_count: Cell<usize>,
    notify: ModelNotify,
}

impl<S, P> ModelChangeListener for ModelInner<S, P>
where
    S: SourceModel,
    P: FixtureInfoProvider,
{
    fn row_added(self: Pin<&Self>, index: usize, count: usize) {
        debug_assert_eq!(
            count, 1,
            "FixtureModel should not add multiple rows at once"
        );
        let fxt_id = self.source_model.get_id(index).expect("todo");

        // TODO: クロスユニバースの処理を考えるとfootprint()よりoccupied_addresses()を使ったほうがいいかも
        let infos = Self::compute_fixture_info(
            self.info_provider.get_universe(fxt_id).unwrap(),
            self.info_provider.get_address(fxt_id).unwrap().value(),
            self.info_provider.get_footprint(fxt_id).unwrap(),
            self.info_provider
                .get_name(fxt_id)
                .unwrap()
                .to_shared_string(),
            self.col_count.get(),
        );

        // Infoの数だけ追加
        let row_order_start = self.row_order.borrow().len();
        let added_rows = infos.len();
        self.row_order.borrow_mut().extend(
            std::iter::repeat_n(fxt_id, added_rows)
                .enumerate()
                .map(|(n, id)| (id, n)),
        );

        self.data.borrow_mut().insert(fxt_id, infos);
        self.notify.row_added(row_order_start, added_rows);
    }

    fn row_changed(self: Pin<&Self>, row: usize) {
        let fxt_id = self.source_model.get_id(row).unwrap();
        let new_infos = Self::compute_fixture_info(
            self.info_provider.get_universe(fxt_id).unwrap(),
            self.info_provider.get_address(fxt_id).unwrap().value(),
            self.info_provider.get_footprint(fxt_id).unwrap(),
            self.info_provider
                .get_name(fxt_id)
                .unwrap()
                .to_shared_string(),
            self.col_count.get(),
        );

        let old_infos_len = self.data.borrow().get(&fxt_id).unwrap().len();
        let pos = self
            .row_order
            .borrow()
            .iter()
            .position(|(id, _)| *id == fxt_id)
            .unwrap();
        self.row_order.borrow_mut().splice(
            pos..pos + old_infos_len,
            std::iter::repeat_n(fxt_id, new_infos.len())
                .enumerate()
                .map(|(n, id)| (id, n)),
        );
        self.data.borrow_mut().insert(fxt_id, new_infos);
        // OPTIM: Shiftが多いのでresetでも大して変わらんかな、と思ったがchanged()のほうがいいのだろうか
        self.notify.reset();
    }

    fn row_removed(self: Pin<&Self>, _index: usize, count: usize) {
        debug_assert_eq!(
            count, 1,
            "FixtureModel should not remove multiple fixtures at once"
        );
        let fxt_id = self.source_model.get_removed_id().unwrap();
        let (first, cnt) = {
            let guard = self.row_order.borrow();
            let iter = guard
                .iter()
                .enumerate()
                .filter(|(_, (id, _))| *id == fxt_id);
            (
                iter.clone().next().map(|(idx, _)| idx).unwrap(),
                iter.count(),
            )
        };

        self.row_order.borrow_mut().drain(first..first + cnt);
        self.data.borrow_mut().remove(&fxt_id);

        self.notify.row_removed(first, cnt);
    }

    fn reset(self: Pin<&Self>) {}
}

impl<S, P> ModelInner<S, P>
where
    S: SourceModel,
{
    fn new(source_model: Rc<S>, doc: P, col_count: usize) -> Self {
        Self {
            source_model,
            info_provider: doc,
            data: RefCell::new(HashMap::new()),
            row_order: RefCell::new(Vec::new()),
            col_count: Cell::new(col_count),
            notify: ModelNotify::default(),
        }
    }

    fn compute_fixture_info(
        universe: UniverseId,
        address: usize,
        footprint: usize,
        name: SharedString,
        col_count: usize,
    ) -> Vec<FixtureInfo> {
        let mut remaining_len = footprint;

        let mut infos = Vec::new();
        loop {
            // FIXME: クロスユニバースのときoverflowする
            let cur_adr = address + footprint - remaining_len;
            let row = cur_adr / col_count;
            let col = cur_adr % col_count - 1;
            // 次の行に行くかどうか
            let length = std::cmp::min(col_count - col, remaining_len);
            let text = if infos.len() == 0 {
                name.clone()
            } else {
                "".to_shared_string()
            };
            infos.push(FixtureInfo {
                universe,
                row,
                col,
                length,
                text,
            });
            if length == remaining_len {
                break;
            } else {
                remaining_len -= col_count - col
            }
        }
        infos
    }
}

#[derive(Debug)]
struct FixtureInfo {
    universe: UniverseId,
    row: usize,
    col: usize,
    length: usize,
    text: SharedString,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tsukuyomidmx_core::{
        doc::{Doc, FakeFixtureDefRegistry},
        fixture::FixtureChange,
        prelude::{DmxAddress, Fixture, FixtureDef, UniverseId},
    };

    const COL_COUNT: usize = 24;

    struct FakeSourceModel {
        data: RefCell<HashMap<FixtureId, Fixture>>,
        keys: RefCell<Vec<FixtureId>>,
        notify: ModelNotify,
        latest_removed: Cell<Option<FixtureId>>,
    }

    impl SourceModel for FakeSourceModel {
        fn get_id(&self, idx: usize) -> Option<FixtureId> {
            self.keys.borrow().get(idx).cloned()
        }

        fn get_removed_id(&self) -> Option<FixtureId> {
            self.latest_removed.get()
        }

        fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId> {
            todo!()
        }

        fn model_tracker(&self) -> &dyn ModelTracker {
            &self.notify
        }
    }

    impl FakeSourceModel {
        fn new() -> Self {
            Self {
                data: RefCell::new(HashMap::new()),
                keys: RefCell::new(Vec::new()),
                notify: ModelNotify::default(),
                latest_removed: Cell::new(None),
            }
        }

        fn add(&self, fxt: Fixture) {
            let fxt_id = fxt.id();
            self.data.borrow_mut().insert(fxt_id, fxt);
            self.keys.borrow_mut().push(fxt_id);
            self.notify.row_added(self.keys.borrow().len() - 1, 1);
        }

        fn update(&self, fxt: Fixture) {
            let fxt_id = fxt.id();
            self.data.borrow_mut().insert(fxt_id, fxt);
            let row = self
                .keys
                .borrow()
                .iter()
                .position(|id| *id == fxt_id)
                .unwrap();
            self.notify.row_changed(row);
        }

        fn remove(&self, fxt_id: FixtureId) {
            let row = self
                .keys
                .borrow()
                .iter()
                .position(|id| *id == fxt_id)
                .unwrap();
            self.keys.borrow_mut().remove(row);
            self.data.borrow_mut().remove(&fxt_id);
            self.latest_removed.set(Some(fxt_id));
            self.notify.row_removed(row, 1);
        }
    }

    struct StubFixtureInfoProvider {
        univ: UniverseId,
        adr: DmxAddress,
        footprint: usize,
        name: SharedString,
    }

    impl StubFixtureInfoProvider {
        fn new(univ: u8, adr: usize, footprint: usize, name: impl ToSharedString) -> Self {
            Self {
                univ: UniverseId::new(univ),
                adr: DmxAddress::new(adr).unwrap(),
                footprint,
                name: name.to_shared_string(),
            }
        }
    }

    impl FixtureInfoProvider for StubFixtureInfoProvider {
        fn get_universe(&self, fxt_id: FixtureId) -> Option<UniverseId> {
            Some(self.univ)
        }

        fn get_address(&self, _fxt_id: FixtureId) -> Option<DmxAddress> {
            Some(self.adr)
        }

        fn get_footprint(&self, _fxt_id: FixtureId) -> Option<usize> {
            Some(self.footprint)
        }

        fn get_name(&self, _fxt_id: FixtureId) -> Option<SharedString> {
            Some(self.name.clone())
        }
    }

    #[test]
    fn model_updates_after_fixture_added() {
        let mut def_rg = FakeFixtureDefRegistry::new();
        let def = FixtureDef::new_dummy();
        let def_id = def.id().to_owned();
        def_rg.insert(def_id.clone(), def);
        // TODO: StubFixtureInfoProviderを使う
        let mut doc = Doc::new_with_def_registry(Box::new(def_rg));
        let source_model = Rc::new(FakeSourceModel::new());

        let model = UniverseViewModel::new(Rc::clone(&source_model), doc.state_view(), COL_COUNT);

        let fxt = Fixture::new(
            "Test Fixture",
            UniverseId::new(0),
            DmxAddress::MIN,
            def_id,
            "4 Channel",
            0.,
            0.,
        );
        let fxt_id = fxt.id();
        doc.add_fixture(fxt.clone()).unwrap();
        source_model.add(fxt);

        let data = model.0.data.borrow();
        let row_order = model.0.row_order.borrow();
        assert_eq!(data.len(), 1);
        let infos = data.get(&fxt_id).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(row_order.len(), 1);
        let info = &infos[0];
        assert_eq!(info.col, 0);
        assert_eq!(info.row, 0);
        assert_eq!(info.length, 4);
        assert_eq!(info.text, "Test Fixture");
    }

    #[test]
    fn model_works_with_overflow() {
        let def = FixtureDef::new_dummy();
        let def_id = def.id().to_owned();
        let source_model = Rc::new(FakeSourceModel::new());
        let stub = StubFixtureInfoProvider::new(0, 22, 4, "Test Fixture");

        let model = UniverseViewModel::new(Rc::clone(&source_model), stub, COL_COUNT);

        let fxt = Fixture::new(
            "Test Fixture",
            UniverseId::new(0),
            DmxAddress::new(22).unwrap(),
            def_id,
            "4 Channel",
            0.,
            0.,
        );
        let fxt_id = fxt.id();
        source_model.add(fxt);

        let data = model.0.data.borrow();
        let row_order = model.0.row_order.borrow();
        assert_eq!(data.len(), 1);
        let infos = data.get(&fxt_id).unwrap();
        assert_eq!(infos.len(), 2);
        assert_eq!(row_order.len(), 2);

        let first = &infos[0];
        assert_eq!(first.col, 21);
        assert_eq!(first.row, 0);
        assert_eq!(first.length, 3);
        assert_eq!(first.text, "Test Fixture");

        let secound = &infos[1];
        assert_eq!(secound.row, 1);
        assert_eq!(secound.col, 0);
        assert_eq!(secound.length, 1);
        assert_eq!(secound.text, "");
    }

    #[test]
    fn model_updates_after_fixture_updated() {
        let mut def_rg = FakeFixtureDefRegistry::new();
        let def = FixtureDef::new_dummy();
        let def_id = def.id().to_owned();
        def_rg.insert(def_id.clone(), def);
        // TODO: StubFixtureInfoProviderを使う
        let mut doc = Doc::new_with_def_registry(Box::new(def_rg));
        let source_model = FixtureModel::create(&mut doc);

        let model = UniverseViewModel::new(Rc::clone(&source_model), doc.state_view(), COL_COUNT);

        let fxt = Fixture::new(
            "Test Fixture",
            UniverseId::new(0),
            DmxAddress::MIN,
            def_id,
            "4 Channel",
            0.,
            0.,
        );

        let fxt_id = fxt.id();
        doc.add_fixture(fxt.clone()).unwrap();

        doc.update_fixture(fxt_id, FixtureChange::Address(DmxAddress::new(10).unwrap()))
            .unwrap();

        let data = model.0.data.borrow();
        let row_order = model.0.row_order.borrow();
        assert_eq!(data.len(), 1);
        assert_eq!(row_order.len(), 1);
        let infos = data.get(&fxt_id).unwrap();
        assert_eq!(infos.len(), 1);
        let info = &infos[0];
        assert_eq!(info.row, 0);
        assert_eq!(info.col, 9);
        assert_eq!(info.length, 4);
        assert_eq!(row_order[0].0, fxt_id);
        assert_eq!(row_order[0].1, 0);
    }

    #[test]
    fn model_updates_after_fixture_removed() {
        let source_model = Rc::new(FakeSourceModel::new());
        let stub = StubFixtureInfoProvider::new(0, 10, 4, "Test Fixture");
        let model = UniverseViewModel::new(Rc::clone(&source_model), stub, COL_COUNT);

        let def_id = FixtureDef::new_dummy().id().to_owned();
        let fxt = Fixture::new(
            "Test Fixture",
            UniverseId::new(0),
            DmxAddress::new(10).unwrap(),
            def_id,
            "4 Channel",
            0.,
            0.,
        );
        let fxt_id = fxt.id();
        source_model.add(fxt);

        source_model.remove(fxt_id);

        let data = model.0.data.borrow();
        let row_order = model.0.row_order.borrow();

        assert_eq!(data.len(), 0);
        assert_eq!(row_order.len(), 0);
    }
}
