use std::sync::Arc;

use slint::{ComponentHandle, Global};
use tracing::trace;
use tsukuyomidmx_core::effects::{EffectChange, FixtureQuery, SimpleEffectBody};

use crate::{
    app::App,
    models::{EffectEditorData, EffectEditorModel},
    ui,
};

pub(super) fn setup(app: &App, effect_model: &EffectEditorModel) {
    let adopter = app.ui.global::<ui::SimpleEffectSingleViewAdopter>();

    adopter.on_update_value({
        let doc = Arc::clone(&app.doc);
        let doc_view = doc.lock().unwrap().state_view();
        let cur_id_state = app.state.read().unwrap().current_effect_id();

        move |offset, value| {
            let offset: usize = offset.try_into().unwrap();
            let cur_id = cur_id_state.get().unwrap().unwrap_effect();

            let new_body = doc_view.with_effects(|it| {
                let fx_body = it.get(&cur_id).unwrap().unwrap_simple();
                let SimpleEffectBody::New { fixtures, values } = fx_body else {
                    unreachable!("this effect is always SimpleEffectBody::New");
                };
                let new_values = values
                    .iter()
                    .map(|((fxt_id, old_offset), old_val)| {
                        if *old_offset == offset {
                            ((*fxt_id, offset), value.try_into().unwrap())
                        } else {
                            ((*fxt_id, *old_offset), *old_val)
                        }
                    })
                    .collect();
                SimpleEffectBody::New {
                    fixtures: fixtures.clone(),
                    values: new_values,
                }
            });
            doc.lock()
                .unwrap()
                .update_effect(cur_id, EffectChange::Simple(new_body))
                .unwrap();
        }
    });

    effect_model.subscribe_current_effect_data({
        // TODO: ここでは別に循環参照にならないかも
        let adopter = adopter.as_weak();

        move |data| {
            let adopter = adopter.unwrap();
            if let Some(EffectEditorData::Simple(data)) = data {
                trace!(component = "SimpleEffectSingleView", "updating effect data");
                adopter.set_fixture_query(data.fixture_query.clone());
                adopter.set_effect_name(data.effect_name.clone());
                adopter.set_item(data.item.clone());
            }
        }
    });
}
