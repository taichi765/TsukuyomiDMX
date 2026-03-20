use std::{cell::RefCell, collections::HashMap, rc::Rc};

use slint::{Model, ModelNotify, SharedString, ToSharedString, VecModel};

use crate::{models::FixtureDefModel, ui};

/// FixtureListViewのModel。
///
/// expandedのみ変更可。変更可能なUI状態とDocStateのprojectionを分離するのが理想だが、
/// .slint側が複雑になるためRust側に複雑性を閉じ込めたほうが良いと判断。
pub struct ManufacturerModel {
    source_model: Rc<FixtureDefModel>,
    expanded_map: RefCell<HashMap<SharedString, bool>>,
    notify: ModelNotify,
}

impl Model for ManufacturerModel {
    type Data = ui::ManufacturerModel;

    fn row_count(&self) -> usize {
        self.source_model.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        // OPTIM: 毎回collectするのはどう考えても遅い、slint::SortModelと同じアプローチを取るべき
        let mut vec = self.source_model.iter().collect::<Vec<_>>();
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        vec.into_iter()
            .nth(row)
            .map(|(manufacturer, defs)| ui::ManufacturerModel {
                expanded: *self
                    .expanded_map
                    .borrow()
                    .get(&manufacturer)
                    .unwrap_or(&false),
                fixtures: Rc::new(VecModel::from(
                    defs.iter()
                        .map(|(id, model_name)| ui::FixtureModel {
                            id: id.to_shared_string(),
                            modes: Rc::new(VecModel::from(vec!["Dummy Mode".into()])).into(), // TODO: 遅延初期化をしたい
                            name: model_name.to_owned(),
                        })
                        .collect::<Vec<_>>(),
                ))
                .into(),
                manufacturer: manufacturer,
            })
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row >= self.row_count() {
            panic!("Model::set_row_data() shouldn't be called with row >= row_count() ")
        }

        let old = self.row_data(row).unwrap();
        if data.manufacturer != old.manufacturer {
            panic!("changing value of manufacturer is not allowed")
        }
        if !data.fixtures.iter().all(|fxt| {
            old.fixtures
                .iter()
                .find(|other| fxt.id == other.id)
                .is_some_and(|old_fxt| {
                    old_fxt.name == fxt.name && old_fxt.modes.iter().eq(fxt.modes.iter())
                })
        }) {
            panic!("changing fixture list is not allowed")
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
    pub fn new(def_model: Rc<FixtureDefModel>) -> Self {
        Self {
            source_model: def_model,
            expanded_map: RefCell::new(HashMap::new()),
            notify: ModelNotify::default(),
        }
    }
}
