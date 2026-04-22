use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use slint::{Model, ModelNotify, ToSharedString};
use tsukuyomidmx_core::{
    doc::{Doc, DocEffect, DocStateView},
    effects::{Effect, EffectBody, EffectSpecId},
    prelude::EffectId,
};

use crate::{app::AnyFunctionId, ui};

pub struct EffectTreeViewModel {
    doc: DocStateView,
    row_order: RefCell<Vec<AnyFunctionId>>,
    data: RefCell<HashMap<AnyFunctionId, ui::EffectTreeViewItemData>>,
    notify: ModelNotify,
}

impl Model for EffectTreeViewModel {
    type Data = ui::EffectTreeViewItemData;

    fn row_count(&self) -> usize {
        self.data.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let guard = self.row_order.borrow();
        let id = guard.get(row)?;
        self.data.borrow().get(id).cloned()
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl EffectTreeViewModel {
    pub fn new(doc: &mut Doc) -> Rc<Self> {
        let row_order = Vec::new();
        let data = doc.state_view().with_effects(|it| {
            it.iter().fold(HashMap::new(), |mut acc, v| {
                acc.insert(
                    AnyFunctionId::Effect(v.0.to_owned()),
                    ui::EffectTreeViewItemData {
                        id: v.0.to_shared_string(),
                        name: v.1.name().to_shared_string(),
                        r#type: ui::FunctionType::Simple,
                    },
                );
                acc
            })
        });
        dbg!(&data);

        let me = Rc::new(Self {
            doc: doc.state_view(),
            row_order: RefCell::new(row_order),
            data: RefCell::new(data),
            notify: ModelNotify::default(),
        });

        doc.subscribe({
            let me_clone = Rc::clone(&me);

            Box::new(move |ev| match ev {
                DocEffect::EffectAdded(id) => me_clone.doc.with_effects(|it| {
                    let fun = it.get(id).unwrap();
                    let id = AnyFunctionId::Effect(*id);
                    let added_row = me_clone.row_order.borrow().len();
                    me_clone.row_order.borrow_mut().push(id);
                    me_clone.data.borrow_mut().insert(
                        id,
                        ui::EffectTreeViewItemData {
                            id: id.to_shared_string(),
                            name: fun.name().to_shared_string(),
                            r#type: get_effect_type(fun),
                        },
                    );
                    me_clone.notify.row_added(added_row, 1);
                }),
                DocEffect::EffectUpdated(id) => {
                    let pos = me_clone
                        .row_order
                        .borrow()
                        .iter()
                        .position(|el| matches!(el, AnyFunctionId::Effect(v) if v==id))
                        .unwrap();
                    me_clone.doc.with_effects(|it| {
                        let fx = it.get(id).unwrap();
                        me_clone.data.borrow_mut().insert(
                            AnyFunctionId::Effect(*id),
                            ui::EffectTreeViewItemData {
                                id: id.to_shared_string(),
                                name: fx.name().to_shared_string(),
                                r#type: get_effect_type(fx),
                            },
                        );
                    });
                    me_clone.notify.row_changed(pos);
                }
                DocEffect::EffectRemoved(id) => {
                    let pos = me_clone
                        .row_order
                        .borrow()
                        .iter()
                        .position(|el| matches!(el, AnyFunctionId::Effect(v) if v==id))
                        .unwrap();
                    me_clone.row_order.borrow_mut().remove(pos);
                    me_clone
                        .data
                        .borrow_mut()
                        .remove(&AnyFunctionId::Effect(*id));
                    me_clone.notify.row_removed(pos, 1);
                }
                DocEffect::EffectSpecAdded(id) => todo!(),
                DocEffect::EffectSpecUpdated(id) => todo!(),
                DocEffect::EffectSpecRemoved(id) => todo!(),
                DocEffect::EffectTemplateAdded(id) => todo!(),
                DocEffect::EffectTemplateUpdated(id) => todo!(),
                DocEffect::EffectTemplateRemoved(id) => todo!(),
                _ => (),
            })
        });

        me
    }

    pub fn get_index(&self, id: AnyFunctionId) -> Option<usize> {
        self.row_order.borrow().iter().position(|v| *v == id)
    }
}

fn get_effect_type(fun: &Effect) -> ui::FunctionType {
    match fun.body() {
        EffectBody::Simple(_) => ui::FunctionType::Simple,
        EffectBody::Sequence(_) => ui::FunctionType::Sequence,
        EffectBody::Parallel(_) => ui::FunctionType::Parallel,
    }
}
