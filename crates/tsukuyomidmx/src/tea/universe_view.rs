use std::rc::Rc;

use slint::{ComponentHandle, Model, ModelExt, ModelRc, VecModel};
use tracing::{instrument, trace_span};
use tsukuyomidmx_core::{doc::DocStateView, prelude::UniverseId};

use crate::{
    app::App,
    models::{FixtureMapModel, FixtureModel, UniverseViewModel},
    ui,
};

#[instrument(skip_all)]
pub fn setup(app: &mut App) {
    let adopter = app.ui.global::<ui::UniverseViewAdopter>();
    let model = Rc::new(UniverseViewModel::new(
        app.shared_model_inner.fixture_model.get().cloned().unwrap(),
        app.doc.lock().unwrap().state_view(),
        24,
    ));
    let cols_model = Rc::new(VecModel::from((0..24).collect::<Vec<i32>>()));
    let rows_model = Rc::new(VecModel::from((0..22).collect::<Vec<i32>>()));

    adopter.set_fixtures(map_model(Rc::clone(&model), UniverseId::new(0)));
    adopter.set_cols(Rc::clone(&cols_model).into());
    adopter.set_rows(Rc::clone(&rows_model).into());
    adopter.on_universe_changed({
        let ui_handle = app.ui.as_weak();
        let model_clone = Rc::clone(&model);
        move |idx| {
            let _span = trace_span!("UniverseView::on_universe_changed", idx).entered();
            let univ_id = if idx == 0 { 0 } else { idx - 1 };
            let univ_id = UniverseId::new(univ_id as u8);
            let model = map_model(Rc::clone(&model_clone), univ_id);
            ui_handle
                .unwrap()
                .global::<ui::UniverseViewAdopter>()
                .set_fixtures(model);
        }
    })
}

/// filter, map, sortなどをしてUIに渡せる状態にする
fn map_model(
    model: Rc<UniverseViewModel<FixtureModel, DocStateView>>,
    univ_id: UniverseId,
) -> ModelRc<ui::UniverseViewFixtureData> {
    // Universeが変わるときどうせ全部resetなので、FilterModelとかを使う必要はない
    let mut model_vec: Vec<_> = model
        .iter()
        .filter(|(univ, _)| *univ == univ_id)
        .map(|(_, data)| data)
        .collect();
    model_vec.sort_by(|a, b| {
        if a.row != b.row {
            a.row.cmp(&b.row)
        } else {
            a.col.cmp(&b.col)
        }
    });
    model_vec.iter_mut().enumerate().for_each(|(idx, data)| {
        if idx % 2 == 1 {
            data.is_odd = true
        }
    });
    Rc::new(VecModel::from(model_vec)).into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_model_maps_correctly() {
        todo!()
    }
}
