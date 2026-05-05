use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Global};
use tracing::{instrument, trace};
use tsukuyomidmx_core::{
    doc::Doc,
    effects::{EffectChange, FixtureQuery, SimpleEffectBody},
};

use crate::{
    Observable,
    app::{AnyEffectId, App},
    models::{EffectEditorData, EffectEditorModel},
    ui,
};

pub(super) fn setup(app: &App, effect_model: &EffectEditorModel) {
    let adopter = app.ui.global::<ui::SimpleEffectSingleViewAdopter>();
    let handler = SimpleEffectSingleViewHandler {
        doc: Arc::clone(&app.doc),
        cur_id_state: app.global_store.read().unwrap().current_effect_id(),
    };

    adopter.on_update_value({
        let handler = handler.clone();

        move |offset, value| {
            handler.update_value(offset, value);
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

trait SimpleEffectSingleViewHandlerTrait {
    fn update_value(&self, offset: i32, value: i32);
}

#[derive(Clone)]
struct SimpleEffectSingleViewHandler {
    doc: Arc<Mutex<Doc>>,
    cur_id_state: Observable<Option<AnyEffectId>>,
}

impl SimpleEffectSingleViewHandlerTrait for SimpleEffectSingleViewHandler {
    #[instrument(skip(self))]
    fn update_value(&self, offset: i32, value: i32) {
        let doc_view = self.doc.lock().unwrap().state_view();
        let offset: usize = offset.try_into().unwrap();
        let cur_id = self.cur_id_state.get().unwrap().unwrap_effect();

        let new_body = doc_view.with_effects(|it| {
            let fx_body = it.get(&cur_id).unwrap().unwrap_simple();
            let SimpleEffectBody::New { fixtures, values } = fx_body else {
                unreachable!("this effect is always SimpleEffectBody::New");
            };
            let new_values = values
                .iter()
                .map(|(&old_offset, &old_val)| {
                    if old_offset == offset {
                        (offset, value.try_into().unwrap())
                    } else {
                        (old_offset, old_val)
                    }
                })
                .collect();
            SimpleEffectBody::New {
                fixtures: fixtures.clone(),
                values: new_values,
            }
        });
        self.doc
            .lock()
            .unwrap()
            .update_effect(cur_id, EffectChange::Simple(new_body))
            .unwrap();
    }
}
