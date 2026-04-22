use std::{
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::{
    app::{AnyFunctionId, App, AppStateChange},
    models::EffectTreeViewModel,
    tea::wrap_callback,
    ui,
};
use slint::{ComponentHandle, Global, ToSharedString};
use tsukuyomidmx_core::{
    effects::{Effect, EffectSpecId},
    prelude::EffectId,
};

#[allow(unused)]
pub fn setup(app: &App) {
    let adopter = app.ui.global::<ui::FunctionListViewAdopter>();
    let model = EffectTreeViewModel::new(&mut app.doc.lock().unwrap());
    adopter.set_model(Rc::clone(&model).into());

    adopter.on_new_simple_function({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback("FunctionListViewAdopter::on_new_simple_function", || {
                doc_clone
                    .lock()
                    .unwrap()
                    .add_effect(Effect::new_simple("Simple Function"))
                    .expect("todo");
            });
        }
    });

    adopter.on_new_sequence_funtion({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback("FunctionListViewAdopter::on_new_sequence_function", || {
                doc_clone
                    .lock()
                    .unwrap()
                    .add_effect(Effect::new_sequence("Sequence Function"))
                    .expect("todo");
            });
        }
    });

    adopter.on_new_parallel_function({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_parallel_function",
                || todo!(),
            );
        }
    });

    adopter.on_new_simple_function_prototype({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_simple_function_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_new_sequence_function_prototype({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_sequence_function_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_new_parallel_function_prototype({
        let doc_clone = app.doc.clone();

        move || {
            wrap_callback(
                "FunctionListViewAdopter::on_new_parallel_function_prototype",
                || todo!(),
            );
        }
    });

    adopter.on_remove_function({
        let doc_clone = app.doc.clone();

        move |id, r#type| {
            wrap_callback("FunctionListViewAdopter::on_remove_function", || {
                if is_prototype(&r#type) {
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

    adopter.on_set_selected_function({
        let adopter_weak = adopter.as_weak();
        let state_weak = app.ui.global::<ui::GlobalState>().as_weak();
        let model_clone = Rc::clone(&model);

        move |id, r#type| {
            wrap_callback("FunctionListViewAdopter::on_set_selected_function", || {
                let id = if is_prototype(&r#type) {
                    AnyFunctionId::Spec(EffectSpecId::from_str(id.as_str()).unwrap())
                } else {
                    AnyFunctionId::Effect(EffectId::from_str(id.as_str()).unwrap())
                };

                let idx = model_clone.get_index(id).unwrap();
                adopter_weak
                    .unwrap()
                    .set_selected_index(idx.try_into().unwrap());
                state_weak
                    .unwrap()
                    .set_current_function(id.to_shared_string());
            });
        }
    });
}

fn is_prototype(val: &ui::FunctionType) -> bool {
    match val {
        ui::FunctionType::SimplePrototype
        | ui::FunctionType::SequencePrototype
        | ui::FunctionType::ParallelPrototype => true,
        ui::FunctionType::Simple | ui::FunctionType::Sequence | ui::FunctionType::Parallel => false,
    }
}
