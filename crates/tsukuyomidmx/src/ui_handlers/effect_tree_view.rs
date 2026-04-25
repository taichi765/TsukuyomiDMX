use std::{
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use super::wrap_callback;
use crate::{
    app::{AnyEffectId, App, AppStateChange},
    models::EffectTreeViewModel,
    ui,
};
use anyhow::Context;
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
                let id:AnyEffectId=serde_json::from_str(&id).with_context(||format!("failed to deserialize id {} from FunctionListViewAdopter::on_remove_effect()",id)).unwrap();
                match id{
                    AnyEffectId::Spec(id)=>{
                        doc_clone.lock().unwrap().remove_effect_spec(id).unwrap();
                    }
                    AnyEffectId::Template(id)=>{
                        doc_clone.lock().unwrap().remove_effect_template(id).unwrap();
                    }
                    AnyEffectId::Effect(id)=>{
                        doc_clone.lock().unwrap().remove_effect(id).unwrap();
                    }
                }
            })
        }
    });

    adopter.on_set_selected_effect({
        let adopter_weak = adopter.as_weak();
        let model_clone = Rc::clone(&model);
        let id_state = app.state.read().unwrap().current_effect_id();

        move |id_str, r#type| {
            wrap_callback("FunctionListViewAdopter::on_set_selected_effect", || {
                let id: AnyEffectId = serde_json::from_str(&id_str).unwrap();

                let idx = model_clone.get_index(id).unwrap();
                adopter_weak
                    .unwrap()
                    .set_selected_index(idx.try_into().unwrap());
                id_state.set(Some(id));
            });
        }
    });
}
