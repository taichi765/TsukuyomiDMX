use std::{cell::Cell, rc::Rc};

use slint::{
    ComponentHandle, FilterModel, MapModel, Model, ModelExt, ModelRc, SharedString, SortModel,
    ToSharedString, VecModel,
};
use tracing::{instrument, trace_span};
use tsukuyomidmx_core::{doc::DocStateView, prelude::UniverseId};

use crate::{
    app::App,
    models::{FixtureMapModel, FixtureModel, UniverseViewModel},
    ui,
};

#[instrument(skip_all)]
pub fn setup(app: &App) {
    let adopter = app.ui.global::<ui::UniverseViewAdopter>();
    let model_base = Rc::new(UniverseViewModel::new(
        app.shared_model_inner.fixture_model.get().cloned().unwrap(),
        app.doc.lock().unwrap().state_view(),
        24,
    ));
    let model = create_mapped_model(Rc::clone(&model_base), UniverseId::MIN);
    let cols_model = Rc::new(VecModel::from((0..24).collect::<Vec<i32>>()));
    let rows_model = Rc::new(VecModel::from((0..22).collect::<Vec<i32>>()));
    let universe_model = Rc::clone(&app.shared_model_inner.universe_model.get().unwrap())
        .map(|u_id| universe_id_to_shared_string(u_id));

    adopter.set_fixtures(Rc::clone(&model).into());
    adopter.set_universes(Rc::new(universe_model).into());
    adopter.set_cols(Rc::clone(&cols_model).into());
    adopter.set_rows(Rc::clone(&rows_model).into());
    adopter.on_universe_changed({
        let ui_handle = app.ui.as_weak();
        move |idx| {
            let _span = trace_span!("UniverseView::on_universe_changed", idx).entered();
            let univ_id = UniverseId::new(idx as u8);
            let model = create_mapped_model(Rc::clone(&model_base), univ_id);
            ui_handle
                .unwrap()
                .global::<ui::UniverseViewAdopter>()
                .set_fixtures(model.into());
        }
    })
}

/// [`UniverseViewModel`]をUIで扱う状態にmapした`Model`を作る
///
/// [`FixtureModel`]が変更されたときはunderlying Modelがよしなに更新してくれる
fn create_mapped_model(
    model: Rc<UniverseViewModel<FixtureModel, DocStateView>>,
    universe: UniverseId,
) -> Rc<
    MapModel<
        SortModel<
            FilterModel<
                Rc<UniverseViewModel<FixtureModel, DocStateView>>,
                impl Fn(&(UniverseId, ui::UniverseViewFixtureData)) -> bool + 'static,
            >,
            impl FnMut(
                &(UniverseId, ui::UniverseViewFixtureData),
                &(UniverseId, ui::UniverseViewFixtureData),
            ) -> std::cmp::Ordering
            + 'static,
        >,
        impl Fn((UniverseId, ui::UniverseViewFixtureData)) -> ui::UniverseViewFixtureData,
    >,
> {
    let sorted_model =
        model
            .filter(move |(u_id, _)| *u_id == universe)
            .sort_by(|(_, a), (_, b)| {
                if a.row != b.row {
                    a.row.cmp(&b.row)
                } else {
                    a.col.cmp(&b.col)
                }
            });

    let next_is_odd = Cell::new(false);
    let model = Rc::new(sorted_model.map(move |(_, data)| {
        let is_odd = next_is_odd.get();
        next_is_odd.set(!is_odd);
        ui::UniverseViewFixtureData {
            row: data.row,
            col: data.col,
            fixture_id: data.fixture_id,
            is_odd,
            length: data.length,
            text: data.text,
        }
    }));
    model
}

/// From/Intoトレイトでやったほうが良さそうだが、Universe名をどう扱うか決まってないのでとりあえずここに
///
/// fixture_list_view.rsにも同じものがあるが、共通とは限らないのでコピペしておく
fn universe_id_to_shared_string(id: UniverseId) -> SharedString {
    format!("Universe {}", id.value() + 1).to_shared_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_model_maps_correctly() {
        todo!()
    }
}
