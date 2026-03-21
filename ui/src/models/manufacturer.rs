use std::{
    cell::{OnceCell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use anyhow::Context;
use derive_getters::Getters;
use slint::{Model, ModelExt, ModelNotify, ModelRc, SharedString, ToSharedString, VecModel};
use thiserror::Error;
use tsukuyomi_core::{
    doc::{DocStateView, FixtureDefLookupError},
    prelude::FixtureDefId,
};

use crate::{models::FixtureDefModel, ui};

/// FixtureListViewのModel。
///
/// expandedのみ変更可。変更可能なUI状態とDocStateのprojectionを分離するのが理想だが、
/// .slint側が複雑になるためRust側に複雑性を閉じ込めたほうが良いと判断。
pub struct ManufacturerModel {
    source_model: Rc<FixtureDefModel>,
    /// sortする必要があるので遅延評価はできなさそう
    inner_data: RefCell<Vec<ManufacturerModelItem>>,
    doc: DocStateView,
    /// manufacturer -> is_expanded
    expanded_map: RefCell<HashMap<SharedString, bool>>,
    notify: ModelNotify,
}

impl Model for ManufacturerModel {
    type Data = ui::ManufacturerModel;

    fn row_count(&self) -> usize {
        self.source_model.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.inner_data
            .borrow()
            .get(row)
            .cloned()
            .map(ManufacturerModelItem::into)
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row >= self.row_count() {
            panic!("Model::set_row_data() shouldn't be called with row >= row_count() ")
        }

        let old = self.row_data(row).unwrap();
        if data.manufacturer != old.manufacturer {
            panic!("changing value of manufacturer from UI is not allowed")
        }
        if !data.fixtures.iter().all(|fxt| {
            old.fixtures
                .iter()
                .find(|other| fxt.id == other.id)
                .is_some_and(|old_fxt| {
                    old_fxt.name == fxt.name && old_fxt.modes.iter().eq(fxt.modes.iter())
                })
        }) {
            panic!("changing fixture list from UI is not allowed")
        };

        self.expanded_map
            .borrow_mut()
            .insert(data.manufacturer, data.expanded);
        self.notify.row_changed(row);
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl ManufacturerModel {
    /// manufacturerの一覧のみを取得する。manufacturerごとのfixture defは[`ManufacturerModel::get_manufacturer_detail()`]を使う。
    pub fn new(def_model: Rc<FixtureDefModel>, doc: DocStateView) -> Self {
        let mut manufacturers = def_model
            .iter()
            .map(|(manufacturer, _)| manufacturer)
            .collect::<Vec<_>>();
        manufacturers.sort();

        let data = manufacturers
            .into_iter()
            .map(|manufacturer| ManufacturerModelItem {
                name: manufacturer,
                expanded: false,
                fixtures: OnceCell::new(),
            })
            .collect();
        Self {
            source_model: def_model,
            doc,
            inner_data: RefCell::new(data),
            expanded_map: RefCell::new(HashMap::new()),
            notify: ModelNotify::default(),
        }
    }

    /// manufacturer内のfixture defの一覧を取得する
    ///
    /// Returns `None()` if manufacturer was not found.
    pub fn get_manufacturer_detail(&self, manufacturer: &str) -> Option<ManufacturerModelItem> {
        let guard = self.inner_data.borrow();
        let m_data = guard.iter().find(|item| item.name == manufacturer)?;

        let fixtures: Vec<_> = self.doc.with_fixture_defs(|it| {
            it.iter_metadata()
                .filter(|e| e.manufacturer == manufacturer)
                .map(|e| FixtureItem {
                    id: e.id.to_owned(),
                    name: e.model.to_shared_string(),
                    modes: OnceCell::new(),
                })
                .collect::<Vec<_>>()
        });
        m_data.fixtures.get_or_init(|| fixtures);
        Some(m_data.to_owned())
    }

    /// defのdetailを取得する。
    pub fn get_fixture_detail(
        &self,
        def: SharedString,
    ) -> Result<FixtureItem, GetFixtureDetailError> {
        let def = FixtureDefId::try_from(def.as_str()).map_err(|e| {
            GetFixtureDetailError::InvalidDefId {
                id: def,
                source_err: e,
            }
        })?;
        let (manufacturer, modes) = self.doc.with_fixture_defs(|it| {
            let def = it
                .get(&def)
                .map_err(|e| GetFixtureDetailError::DefNotFound {
                    id: def.clone(),
                    source: e,
                })?;
            let modes = def
                .modes_all()
                .keys()
                .map(|name| name.to_shared_string())
                .collect::<Vec<_>>();
            Ok((def.manufacturer().to_shared_string(), modes))
        })?;

        let guard = self.inner_data.borrow();
        let m_data = guard.iter().find(|item| item.name == manufacturer).unwrap();
        let fxt_data = m_data
            .fixtures
            .get()
            .expect(
                "update_fixture_detail() shouln't be called before update_manufacturer_detail()",
            )
            .iter()
            .find(|item| item.id == def)
            .unwrap();
        fxt_data
            .modes
            .get_or_init(|| Rc::new(VecModel::from(modes)));
        Ok(fxt_data.to_owned())
    }

    /// Just toggle `expanded` property.
    ///
    /// Returns new value of `expanded` if manufacturer present, or `None()` if not.
    pub fn toggle_expanded(&self, manufacturer: &str) -> Option<bool> {
        let mut guard = self.inner_data.borrow_mut();
        let (row, m_data) = guard
            .iter_mut()
            .enumerate()
            .find(|(_, item)| &item.name == manufacturer)
            .unwrap();
        let ret = !m_data.expanded;
        m_data.expanded = ret;

        drop(guard); // drop before notify
        self.notify.row_changed(row);
        Some(ret)
    }
}

#[derive(Clone, Getters)]
pub struct ManufacturerModelItem {
    name: SharedString,
    expanded: bool,
    fixtures: OnceCell<Vec<FixtureItem>>,
}

impl Into<ui::ManufacturerModel> for ManufacturerModelItem {
    fn into(self) -> ui::ManufacturerModel {
        ui::ManufacturerModel {
            manufacturer: self.name,
            expanded: self.expanded,
            fixtures: Rc::new(VecModel::from(
                self.fixtures
                    .get()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .cloned()
                    .map(FixtureItem::into)
                    .collect::<Vec<_>>(),
            ))
            .into(),
        }
    }
}

#[derive(Clone, Getters)]
pub struct FixtureItem {
    id: FixtureDefId,
    name: SharedString,
    #[getter(skip)]
    modes: OnceCell<Rc<VecModel<SharedString>>>,
}

impl FixtureItem {
    pub fn modes(&self) -> Rc<VecModel<SharedString>> {
        Rc::clone(&self.modes.get().unwrap()) // get_fixture_detail()以外からFixtureItemを作ることはない
    }
}

impl Into<ui::FixtureModel> for FixtureItem {
    fn into(self) -> ui::FixtureModel {
        ui::FixtureModel {
            id: self.id.to_shared_string(),
            name: self.name,
            modes: self
                .modes
                .get()
                .cloned()
                .unwrap_or(Rc::new(VecModel::from(Vec::new())))
                .into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum GetFixtureDetailError {
    #[error("invalid fixture def id: {id}: {source_err}")]
    InvalidDefId {
        id: SharedString,
        source_err: String, // can't be used as #[source]
    },
    #[error("cannot find fixture def {id}: {source:?}")]
    DefNotFound {
        id: FixtureDefId,
        source: FixtureDefLookupError,
    },
}

#[cfg(test)]
mod tests {}
