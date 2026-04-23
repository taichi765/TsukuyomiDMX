use std::{
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    app::{AnyEffectId, App, AppStateChange},
    models::EffectTreeViewModel,
    tea::wrap_callback,
    ui,
};
use slint::{ComponentHandle, Global, SharedString, ToSharedString};
use tsukuyomidmx_core::{
    effects::{Effect, EffectSpecId},
    prelude::EffectId,
};

#[allow(unused)]
pub fn setup(app: &App) {
    let adopter = app.ui.global::<ui::EffectTreeViewAdopter>();
    let model = EffectTreeViewModel::new(&mut app.doc.lock().unwrap());
    adopter.set_model(Rc::clone(&model).into());

    adopter.on_new_simple_effect({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback("FunctionListViewAdopter::on_new_simple_effect", || {
                doc_clone
                    .lock()
                    .unwrap()
                    .add_effect(Effect::new_simple("Simple Function"))
                    .expect("todo");
            });
        }
    });

    adopter.on_new_sequence_effect({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback("FunctionListViewAdopter::on_new_sequence_effect", || {
                doc_clone
                    .lock()
                    .unwrap()
                    .add_effect(Effect::new_sequence("Sequence Function"))
                    .expect("todo");
            });
        }
    });

    adopter.on_new_parallel_effect({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_parallel_effect",
                || todo!(),
            );
        }
    });

    adopter.on_new_simple_effect_spec({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_simple_effect_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_new_sequence_effect_spec({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_sequence_effect_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_new_parallel_effect_spec({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_parallel_effect_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_remove_effect({
        let doc_clone = app.doc.clone();

        move |id, r#type| {
            wrap_callback("FunctionListViewAdopter::on_remove_effect", || {
                if is_effect_spec(&r#type) {
                    let id = EffectId::from_str(id.as_str()).expect("todo");
                    doc_clone.lock().unwrap().remove_effect(id).expect("todo");
                } else {
                    let id = EffectSpecId::from_str(id.as_str()).expect("todo");
                    doc_clone
                        .lock()
                        .unwrap()
                        .remove_effect_spec(id)
                        .expect("todo");
                }
            })
        }
    });

    adopter.on_set_selected_effect({
        let adopter_weak = adopter.as_weak();
        let state_weak = app.ui.global::<ui::GlobalState>().as_weak();
        let model_clone = Rc::clone(&model);

        move |id, r#type| {
            wrap_callback("FunctionListViewAdopter::on_set_selected_effect", || {
                let id = if is_effect_spec(&r#type) {
                    AnyEffectId::Spec(EffectSpecId::from_str(id.as_str()).unwrap())
                } else {
                    AnyEffectId::Effect(EffectId::from_str(id.as_str()).unwrap())
                };

                let idx = model_clone.get_index(id).unwrap();
                adopter_weak
                    .unwrap()
                    .set_selected_index(idx.try_into().unwrap());
                state_weak
                    .unwrap()
                    .set_current_effect_id(id.to_shared_string());
            });
        }
    });
}

trait ToAnyEffectId {
    fn to_any_effect_id(&self, val: SharedString) -> AnyEffectId;
}

impl ToAnyEffectId for ui::EffectKind {
    fn to_any_effect_id(&self) -> AnyEffectId {
        match self {
            ui::EffectKind::SimpleSpec
            | ui::EffectKind::SequenceSpec
            | ui::EffectKind::ParallelSpec => todo!(),
        }
    }
}
