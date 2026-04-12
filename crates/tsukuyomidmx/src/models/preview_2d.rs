use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc, sync::mpsc::Receiver};

use i_slint_core::model::{ModelChangeListener, ModelChangeListenerContainer};
use slint::{Color, LogicalPosition, Model, ModelNotify, ModelTracker, ToSharedString};
use tsukuyomidmx_core::{
    doc::DocStateView,
    plugins::DmxFrame,
    prelude::{Capability, CapabilityKind, DmxAddress, FixtureId, UniverseId},
};

use crate::{colors::ColorInfo, models::FixtureModel, ui};

pub struct Preview2DModel<S: SourceModel, P: FixtureInfoProvider>(
    Pin<Box<ModelChangeListenerContainer<ModelInner<S, P>>>>,
);

pub trait SourceModel {
    fn get_id(&self, idx: usize) -> Option<FixtureId>;
    fn get_removed_id(&self) -> Option<FixtureId>;
    fn get_index(&self, id: FixtureId) -> Option<usize>;
    fn model_tracker(&self) -> &dyn ModelTracker;
}

impl SourceModel for FixtureModel {
    fn get_id(&self, idx: usize) -> Option<FixtureId> {
        self.with_row(idx, |fxt| fxt.id())
    }

    fn get_index(&self, id: FixtureId) -> Option<usize> {
        self.get_index(id)
    }

    fn get_removed_id(&self) -> Option<FixtureId> {
        self.latest_removed()
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        self.model_tracker()
    }
}

pub trait FixtureInfoProvider {
    fn get_position(&self, fxt_id: FixtureId) -> Option<LogicalPosition>;

    fn resolve_channel_capability(
        &self,
        univ: UniverseId,
        address: DmxAddress,
        value: u8,
    ) -> Option<(FixtureId, CapabilityKind)>;

    fn iter_fixtures_and_positions(&self) -> impl Iterator<Item = (FixtureId, LogicalPosition)>;
}

impl FixtureInfoProvider for DocStateView {
    fn get_position(&self, fxt_id: FixtureId) -> Option<LogicalPosition> {
        self.with_fixtures(|it| {
            let fxt = it.get(&fxt_id)?;
            Some(LogicalPosition {
                x: fxt.x(),
                y: fxt.y(),
            })
        })
    }

    fn resolve_channel_capability(
        &self,
        univ: UniverseId,
        address: DmxAddress,
        _value: u8,
    ) -> Option<(FixtureId, CapabilityKind)> {
        let (fxt_id, offset) = self.with_address_index(|it| it.get(univ, address).cloned())?;
        let cap = self.with_fixtures_and_defs(|fxts, defs| {
            let fxt = fxts.get(&fxt_id).unwrap();
            let def = defs.get(fxt.fixture_def()).unwrap();
            let channel_name = def
                .mode(fxt.fixture_mode())
                .unwrap()
                .get_channel_by_offset(offset)
                .unwrap();
            let chanel = def.channel_template(channel_name).unwrap();
            chanel.capability().clone()
        });
        let kind = match cap {
            Capability::Single(kind) => kind,
            Capability::Multiple(_kinds) => todo!(),
        };
        Some((fxt_id, kind))
    }

    fn iter_fixtures_and_positions(&self) -> impl Iterator<Item = (FixtureId, LogicalPosition)> {
        self.with_fixtures(|it| {
            it.iter()
                .map(|(fxt_id, fxt)| {
                    (
                        fxt_id.clone(),
                        LogicalPosition {
                            x: fxt.x(),
                            y: fxt.y(),
                        },
                    )
                })
                .collect::<Vec<_>>()
            // 基本的に初期化時にしか呼ばれないので多少無駄が多くても問題ない
        })
        .into_iter()
    }
}

impl<S, P> Model for Preview2DModel<S, P>
where
    S: SourceModel + 'static,
    P: FixtureInfoProvider + 'static,
{
    type Data = ui::FixtureEntityData;

    fn row_count(&self) -> usize {
        self.0.data.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let id = self.0.source_model.get_id(row)?;
        self.0.data.borrow().get(&id).cloned()
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.0.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl<S, P> Preview2DModel<S, P>
where
    S: SourceModel + 'static,
    P: FixtureInfoProvider + 'static,
{
    pub fn new(
        source_model: Rc<S>,
        info_provider: P,
        frame_rx: Receiver<(UniverseId, DmxFrame)>,
    ) -> Self {
        let inner = Box::pin(ModelChangeListenerContainer::new(ModelInner::new(
            Rc::clone(&source_model),
            info_provider,
            frame_rx,
        )));

        source_model
            .model_tracker()
            .attach_peer(inner.as_ref().model_peer());
        Self(inner)
    }
    pub fn update(&self) {
        self.0.update()
    }
}

/// FixtureModelのModelChangeListener。
///
/// row orderはFixtureModelを参照するがModelNotifyは自前 (1:1対応かつcolorとかはDoc関係ないので)
struct ModelInner<S, P> {
    source_model: Rc<S>,
    info_provider: P,
    // TODO: ui::XxxDataとdataで意味が違うのに同じ単語使ってるのモヤる
    data: RefCell<HashMap<FixtureId, ui::FixtureEntityData>>,
    frame_rx: Receiver<(UniverseId, DmxFrame)>,
    notify: ModelNotify,
}

impl<S, P> ModelInner<S, P>
where
    S: SourceModel,
    P: FixtureInfoProvider,
{
    pub fn new(
        source_model: Rc<S>,
        info_provider: P,
        frame_rx: Receiver<(UniverseId, DmxFrame)>,
    ) -> Self {
        let data = info_provider
            .iter_fixtures_and_positions()
            .map(|(fxt_id, pos)| {
                (
                    fxt_id,
                    ui::FixtureEntityData {
                        fixture_id: fxt_id.to_shared_string(),
                        color: Color::default(),
                        pos,
                    },
                )
            })
            .collect();
        Self {
            source_model,
            info_provider,
            data: RefCell::new(data),
            frame_rx,
            notify: ModelNotify::default(),
        }
    }

    /// slint::Timerなどで繰り返し呼ばれるやつ
    fn update(&self) {
        while let Ok((univ, frame)) = self.frame_rx.try_recv() {
            self.apply_dmx_frame(univ, frame);
        }
    }

    /// Actually applies dmx frame to UI's model.
    fn apply_dmx_frame(&self, univ: UniverseId, dmx_data: DmxFrame) {
        let fxt_color_map = dmx_data.iter().fold(HashMap::new(), |mut acc, (adr, val)| {
            if let Some((fxt_id, cap)) = self
                .info_provider
                .resolve_channel_capability(univ, adr, val)
            {
                set_color(fxt_id, &mut acc, cap, val);
            }
            acc
        });

        fxt_color_map.into_iter().for_each(|(fxt_id, color)| {
            let mut guard = self.data.borrow_mut();
            let data = guard.get_mut(&fxt_id).unwrap();
            data.color = color.to_slint_color();
            let idx = self.source_model.get_index(fxt_id).unwrap();
            self.notify.row_changed(idx);
        });
    }
}

impl<S, P> ModelChangeListener for ModelInner<S, P>
where
    S: SourceModel,
    P: FixtureInfoProvider,
{
    fn row_added(self: Pin<&Self>, index: usize, count: usize) {
        debug_assert_eq!(1, count, "multiple fixtures should not be added at once");
        let fxt_id = self.source_model.get_id(index).unwrap();
        self.data.borrow_mut().insert(
            fxt_id,
            ui::FixtureEntityData {
                color: Color::default(),
                fixture_id: fxt_id.to_shared_string(),
                pos: self.info_provider.get_position(fxt_id).unwrap(),
            },
        );
        self.notify.row_added(index, 1);
    }

    fn row_changed(self: Pin<&Self>, row: usize) {
        let fxt_id = self.source_model.get_id(row).unwrap();
        let new_pos = self.info_provider.get_position(fxt_id).unwrap();
        let mut guard = self.data.borrow_mut();
        let data = guard.get_mut(&fxt_id).unwrap();
        if data.pos != new_pos {
            data.pos = new_pos;
            self.notify.row_changed(row);
        }
    }

    fn row_removed(self: Pin<&Self>, index: usize, count: usize) {
        debug_assert_eq!(1, count, "multiple fixtures should not be removed at once");
        let fxt_id = self.source_model.get_removed_id().unwrap();
        self.data.borrow_mut().remove(&fxt_id);
        self.notify.row_removed(index, 1);
    }

    fn reset(self: Pin<&Self>) {}
}

/// Helper function to set color based on `CapabilityKind`.
fn set_color(
    fxt_id: FixtureId,
    map: &mut HashMap<FixtureId, ColorInfo>,
    kind: CapabilityKind,
    value: u8,
) {
    let color = map.entry(fxt_id).or_default();
    match kind {
        CapabilityKind::Intensity => color.dimmer = value,
        CapabilityKind::Red => color.red = value,
        CapabilityKind::Green => color.green = value,
        CapabilityKind::Blue => color.blue = value,
        CapabilityKind::White => color.white = value,
        CapabilityKind::Amber => color.amber = value,
        CapabilityKind::UV => color.uv = value,
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc, sync::mpsc};

    use tsukuyomidmx_core::prelude::{Fixture, FixtureDef};

    use crate::models::test_helpers::{DummyModelChangeEvent, SpyModelPeer};

    use super::*;
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

        fn get_index(&self, fxt_id: FixtureId) -> Option<usize> {
            self.keys.borrow().iter().position(|id| *id == fxt_id)
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
        caps: HashMap<(UniverseId, DmxAddress), (FixtureId, CapabilityKind)>,
        positions: HashMap<FixtureId, LogicalPosition>,
    }

    impl StubFixtureInfoProvider {
        fn new(
            caps: impl Iterator<Item = (UniverseId, DmxAddress, FixtureId, CapabilityKind)>,
            positions: impl Iterator<Item = (FixtureId, LogicalPosition)>,
        ) -> Self {
            Self {
                caps: caps
                    .map(|(univ, adr, fxt_id, cap)| ((univ, adr), (fxt_id, cap)))
                    .collect(),
                positions: positions.collect(),
            }
        }
    }

    impl FixtureInfoProvider for StubFixtureInfoProvider {
        fn get_position(&self, fxt_id: FixtureId) -> Option<LogicalPosition> {
            self.positions.get(&fxt_id).copied()
        }

        fn resolve_channel_capability(
            &self,
            univ: UniverseId,
            address: DmxAddress,
            _value: u8,
        ) -> Option<(FixtureId, CapabilityKind)> {
            self.caps.get(&(univ, address)).cloned()
        }

        fn iter_fixtures_and_positions(
            &self,
        ) -> impl Iterator<Item = (FixtureId, LogicalPosition)> {
            self.positions
                .iter()
                .map(|(id, pos)| (id.clone(), pos.clone()))
        }
    }

    #[test]
    fn model_applies_frame_correctly_and_notify_change() {
        let (frame_tx, frame_rx) = mpsc::channel();

        let def = FixtureDef::new_dummy();
        let fxt = Fixture::new(
            "Test Fixture",
            UniverseId::new(0),
            DmxAddress::new(22).unwrap(),
            def.id().to_owned(),
            "4 Channel",
            0.,
            0.,
        );
        let fxt_id = fxt.id();
        let source_model = Rc::new(FakeSourceModel::new());
        source_model.add(fxt);

        let stub = StubFixtureInfoProvider::new(
            vec![
                (
                    UniverseId::new(0),
                    DmxAddress::new(1).unwrap(),
                    fxt_id,
                    CapabilityKind::Intensity,
                ),
                (
                    UniverseId::new(0),
                    DmxAddress::new(2).unwrap(),
                    fxt_id,
                    CapabilityKind::Red,
                ),
            ]
            .into_iter(),
            vec![(fxt_id, LogicalPosition { x: 10., y: 20. })].into_iter(),
        );
        let model = Preview2DModel::new(Rc::clone(&source_model), stub, frame_rx);
        let container = Box::pin(ModelChangeListenerContainer::new(SpyModelPeer::new()));
        model
            .model_tracker()
            .attach_peer(container.as_ref().model_peer());

        let mut frame = [0; 512];
        frame[0] = 255;
        frame[1] = 200;
        frame_tx.send((UniverseId::new(0), frame.into())).unwrap();

        model.update();

        let guard = model.0.data.borrow();
        let data = guard.get(&fxt_id).unwrap();
        assert_eq!(data.pos, LogicalPosition { x: 10., y: 20. });
        assert_eq!(data.fixture_id, fxt_id.to_shared_string());
        assert_eq!(data.color.red(), 200);

        assert_eq!(container.events.borrow().len(), 1);
        assert_eq!(
            container.events.borrow()[0],
            DummyModelChangeEvent::Changed(0)
        );
    }
}
