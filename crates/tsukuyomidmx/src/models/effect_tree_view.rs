use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use slint::{Model, ModelNotify, ToSharedString};
use tsukuyomidmx_core::{
    doc::{Doc, DocEffect, DocStateView},
    effects::{
        Effect, EffectBody, EffectSpec, EffectSpecBody, EffectSpecId, EffectTemplate,
        EffectTemplateBody,
    },
    prelude::EffectId,
};

use crate::{app::AnyEffectId, ui};

pub struct EffectTreeViewModel {
    doc: DocStateView,
    row_order: RefCell<Vec<AnyEffectId>>,
    data: RefCell<HashMap<AnyEffectId, ui::EffectTreeViewItemData>>,
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
                    AnyEffectId::Effect(v.0.to_owned()),
                    ui::EffectTreeViewItemData {
                        id: serde_json::to_string(&v.0).unwrap().to_shared_string(),
                        name: v.1.name().to_shared_string(),
                        kind: ui::EffectKind::Simple,
                        indent: 0, //TODO
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
                    let effect = it.get(id).unwrap();
                    let id = AnyEffectId::Effect(*id);
                    let added_row = me_clone.row_order.borrow().len();
                    me_clone.row_order.borrow_mut().push(id);
                    me_clone.data.borrow_mut().insert(
                        id,
                        ui::EffectTreeViewItemData {
                            id: serde_json::to_string(&id).unwrap().to_shared_string(),
                            name: effect.name().to_shared_string(),
                            kind: effect.effect_kind(),
                            indent: 0, //TODO
                        },
                    );
                    me_clone.notify.row_added(added_row, 1);
                }),
                DocEffect::EffectUpdated(id) => {
                    let pos = me_clone
                        .row_order
                        .borrow()
                        .iter()
                        .position(|el| matches!(el, AnyEffectId::Effect(v) if v==id))
                        .unwrap();
                    me_clone.doc.with_effects(|it| {
                        let fx = it.get(id).unwrap();
                        me_clone.data.borrow_mut().insert(
                            AnyEffectId::Effect(*id),
                            ui::EffectTreeViewItemData {
                                id: serde_json::to_string(id).unwrap().to_shared_string(),
                                name: fx.name().to_shared_string(),
                                kind: fx.effect_kind(),
                                indent: 0, //TODO
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
                        .position(|el| matches!(el, AnyEffectId::Effect(v) if v==id))
                        .unwrap();
                    me_clone.row_order.borrow_mut().remove(pos);
                    me_clone.data.borrow_mut().remove(&AnyEffectId::Effect(*id));
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

    pub fn get_index(&self, id: AnyEffectId) -> Option<usize> {
        self.row_order.borrow().iter().position(|v| *v == id)
    }
}

trait GetEffectKind {
    fn effect_kind(&self) -> ui::EffectKind;
}

impl GetEffectKind for Effect {
    fn effect_kind(&self) -> ui::EffectKind {
        match self.body() {
            EffectBody::Simple(_) => ui::EffectKind::Simple,
            EffectBody::Sequence(_) => ui::EffectKind::Sequence,
            EffectBody::Parallel(_) => ui::EffectKind::Parallel,
        }
    }
}

impl GetEffectKind for EffectSpec {
    fn effect_kind(&self) -> ui::EffectKind {
        match self.body() {
            EffectSpecBody::Simple(_) => ui::EffectKind::SimpleSpec,
            EffectSpecBody::Sequence(_) => ui::EffectKind::SequenceSpec,
            EffectSpecBody::Parallel(_) => ui::EffectKind::ParallelSpec,
        }
    }
}

impl GetEffectKind for EffectTemplate {
    fn effect_kind(&self) -> ui::EffectKind {
        match self.body() {
            EffectTemplateBody::Simple(_) => ui::EffectKind::SimpleTemplate,
            EffectTemplateBody::Sequence(_) => ui::EffectKind::SequenceTemplate,
            EffectTemplateBody::Parallel(_) => ui::EffectKind::ParallelTemplate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_tree_view_model_adds_item_on_doc_effect() {
        todo!()
    }
}
