use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::{Model, ModelNotify, VecModel};
use tsukuyomidmx_core::{
    doc::{Doc, DocEffect},
    prelude::UniverseId,
};

pub struct UniverseModel(VecModel<UniverseId>);

impl Model for UniverseModel {
    type Data = UniverseId;

    fn row_count(&self) -> usize {
        self.0.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.0.row_data(row)
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        self.0.model_tracker()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl Debug for UniverseModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec = self.0.iter().collect::<Vec<_>>();
        vec.fmt(f)
    }
}

impl UniverseModel {
    pub fn new(doc: &mut Doc) -> Rc<Self> {
        let universes = doc.state_view().universes();
        let me = Rc::new(Self(VecModel::from(universes)));

        doc.subscribe({
            let doc_view = doc.state_view();
            let me_clone = Rc::clone(&me);
            Box::new(move |ef| match ef {
                DocEffect::UniverseAdded(_) | DocEffect::UniverseRemoved(_) => {
                    let mut new = doc_view.universes();
                    new.sort();
                    me_clone.0.set_vec(new);
                }
                _ => (),
            })
        });
        me
    }
}
