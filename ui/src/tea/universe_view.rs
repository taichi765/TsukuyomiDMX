use std::rc::Rc;

use slint::{ComponentHandle, VecModel};

use crate::{
    app::App,
    models::{FixtureMapModel, UniverseViewModel},
    ui,
};

pub fn setup(app: &mut App) {
    let adopter = app.ui.global::<ui::UniverseViewAdopter>();
    let model = Rc::new(UniverseViewModel::new(
        app.shared_model_inner.fixture_model.get().cloned().unwrap(),
        app.doc.lock().unwrap().state_view(),
        24,
    ));
    let cols_model = Rc::new(VecModel::from((0..24).collect::<Vec<i32>>()));
    let rows_model = Rc::new(VecModel::from((0..22).collect::<Vec<i32>>()));

    adopter.set_fixtures(Rc::clone(&model).into());
    adopter.set_cols(Rc::clone(&cols_model).into());
    adopter.set_rows(Rc::clone(&rows_model).into());
}

#[cfg(test)]
mod tests {}
