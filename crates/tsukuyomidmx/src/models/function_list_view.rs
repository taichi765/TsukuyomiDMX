use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use slint::{Model, ModelNotify, ToSharedString};
use tsukuyomidmx_core::{
    doc::{Doc, DocEffect, DocStateView},
    functions::{Function, FunctionBody, FunctionPrototypeId},
    prelude::AppliedFunctionId,
};

use crate::ui;

pub struct FunctionListViewModel {
    doc: DocStateView,
    row_order: RefCell<Vec<AnyFunctionId>>,
    data: RefCell<HashMap<AnyFunctionId, ui::FunctionData>>,
    notify: ModelNotify,
}

impl Model for FunctionListViewModel {
    type Data = ui::FunctionData;

    fn row_count(&self) -> usize {
        self.data.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        todo!()
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        todo!()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl FunctionListViewModel {
    pub fn new(doc: &mut Doc) -> Rc<Self> {
        let row_order = Vec::new();
        let data = HashMap::new();

        let me = Rc::new(Self {
            doc: doc.state_view(),
            row_order: RefCell::new(row_order),
            data: RefCell::new(data),
            notify: ModelNotify::default(),
        });

        doc.subscribe({
            let me_clone = Rc::clone(&me);

            Box::new(move |ev| match ev {
                DocEffect::FunctionAdded(id) => me_clone.doc.with_functions(|it| {
                    let fun = it.get(id).unwrap();
                    let id = AnyFunctionId::Applied(*id);
                    let added_row = me_clone.row_order.borrow().len();
                    me_clone.row_order.borrow_mut().push(id);
                    me_clone.data.borrow_mut().insert(
                        id,
                        ui::FunctionData {
                            id: id.to_shared_string(),
                            name: fun.name().to_shared_string(),
                            r#type: get_function_type(fun),
                        },
                    );
                    me_clone.notify.row_added(added_row, 1);
                }),
                DocEffect::FunctionUpdated(id) => todo!(),
                DocEffect::FunctionRemoved(id) => {
                    let pos = me_clone
                        .row_order
                        .borrow()
                        .iter()
                        .position(|el| matches!(el, AnyFunctionId::Applied(v) if v==id))
                        .unwrap();
                    me_clone.row_order.borrow_mut().remove(pos);
                    me_clone
                        .data
                        .borrow_mut()
                        .remove(&AnyFunctionId::Applied(*id));
                    me_clone.notify.row_removed(pos, 1);
                }
                DocEffect::FunctionPrototypeAdded(id) => todo!(),
                DocEffect::FunctionPrototypeUpdated(id) => todo!(),
                DocEffect::FunctionPrototypeRemoved(id) => todo!(),
                _ => (),
            })
        });

        me
    }
}

fn get_function_type(fun: &Function) -> ui::FunctionType {
    match fun.body() {
        FunctionBody::Simple(_) => ui::FunctionType::Simple,
        FunctionBody::Sequence(_) => ui::FunctionType::Sequence,
        FunctionBody::Parallel(_) => ui::FunctionType::Parallel,
    }
}
