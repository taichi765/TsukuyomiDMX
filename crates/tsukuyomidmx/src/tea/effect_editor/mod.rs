mod simple_effect_single_view;

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::ComponentHandle;

use crate::{app::App, models::EffectEditorModel, ui};

pub fn setup(app: &App) {
    let model = EffectEditorModel::new(
        Arc::clone(&app.doc),
        app.state.read().unwrap().current_effect_id(),
    );

    let adopter = app.ui.global::<ui::EffectEditorPanelAdopter>();
    adopter.set_model(model.tabs_model());
    adopter.on_close({
        let model = Rc::clone(&model);
        move |_id| {
            model.close_tab(0); // TODO: idではなくidxを受け取る
        }
    });

    simple_effect_single_view::setup(app, &model);
}
