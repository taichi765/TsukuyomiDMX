use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::{ComponentHandle, MapModel, Model, ToSharedString, VecModel};
use tracing::debug;
use tsukuyomi_core::{
    doc::{
        Doc, DocStateView, FixtureAddError, FixtureDefNotFoundError, ModeNotFoundError,
        ValidateError,
    },
    prelude::{DmxAddress, Fixture, FixtureDefId, UniverseId},
};
use uuid::Uuid;

use crate::{
    app::{App, AppAction, Dispatcher},
    models::{FixtureDefModel, ManufacturerModel},
    ui,
};

pub fn setup(app: &mut App) {
    let doc_view = app.doc.lock().unwrap().state_view();
    let doc_clone = Arc::clone(&app.doc);
    let adopter = app.ui.global::<ui::FixtureListAdopter>();

    let def_model = FixtureDefModel::create(&mut app.doc.lock().unwrap());
    app.shared_model_inner
        .def_model
        .set(Rc::clone(&def_model))
        .unwrap();
    let manufacturer_model = Rc::new(ManufacturerModel::new(
        def_model,
        app.doc.lock().unwrap().state_view(),
    ));

    adopter.set_model(Rc::clone(&manufacturer_model).into());

    adopter.on_patch({
        let doc_view_clone = doc_view.clone();

        move |universe, address, fixture_def_id, mode, pos| {
            let universe_id = parse_universe_id(universe.as_str());
            let def_id = FixtureDefId::try_from(fixture_def_id.as_str()).unwrap();
            let default_fxt_name = {
                let model_name = doc_view_clone
                    .with_fixture_defs(|it| it.get(&def_id).expect("todo").model().to_string());
                let num = 0; // TODO: 同じFixtureDefを使うFixtureの数を取得する(DocStoreに追加？)
                format!("{}({})", model_name, num)
            };

            let new_fxt = Fixture::new(
                default_fxt_name,
                universe_id,
                DmxAddress::new(address as usize).unwrap(),
                def_id,
                mode.to_string(),
                pos.x,
                pos.y,
            );

            let result = doc_clone.lock().unwrap().add_fixture(new_fxt);
            match result {
                Ok(_) => "".to_shared_string(),
                Err(e) => {
                    match e {
                        FixtureAddError::FixtureDefNotFound(FixtureDefNotFoundError {
                            fixture_id: _,
                            fixture_def_id: def_id,
                            source: e,
                        }) => format!("couldn't find fixture definition {}: {:?}", def_id, e)
                            .to_shared_string(),
                        FixtureAddError::ModeNotFound(ModeNotFoundError {
                            fixture_def: def_id,
                            mode,
                        }) => format!("there's no mode {mode} in the definition {def_id:?}")
                            .to_shared_string(),
                        FixtureAddError::AddressValidateError(
                            ValidateError::AddressConflicted(conflicts),
                        ) => {
                            format!("{} addresses conflicted: {:?}", conflicts.len(), conflicts)
                                .to_shared_string() // TODO: 見やすくする
                        }
                        FixtureAddError::FixtureAlreadyExists(fxt_id) => {
                            format!("fixture id {fxt_id:?} already exists").to_shared_string()
                        }
                    }
                }
            }
        }
    });

    adopter.on_toggle_expand_manufacturer({
        let model = Rc::clone(&manufacturer_model);
        move |name| {
            let now = std::time::Instant::now();
            let model = Rc::clone(&model);
            slint::spawn_local(async move {
                model.get_manufacturer_detail(&name).unwrap();
                model.toggle_expanded(&name);
            })
            .unwrap();
            let elapsed = now.elapsed();
            debug!(
                callback = "FixtureListViewAdopter::toggle-expand-manufacturer",
                elapsed = ?elapsed
            );
        }
    });

    adopter.on_update_current_fixture_modes({
        let ui_handle = app.ui.as_weak();
        let model = Rc::clone(&manufacturer_model);
        move |def_id| match model.get_fixture_detail(def_id) {
            Ok(fxt_data) => {
                ui_handle
                    .unwrap()
                    .global::<ui::FixtureListAdopter>()
                    .set_current_fixture_modes(fxt_data.modes().into());
            }
            Err(_) => {
                // TODO: improve error message
                ui_handle
                    .unwrap()
                    .set_error_message("failed to load definition".to_shared_string());
            }
        }
    });

    adopter.on_get_next_address(move |universe| {
        let uni_id = parse_universe_id(&universe);
        let max = doc_view.current_max_address(uni_id);
        match max {
            Some(adr) => {
                if adr == DmxAddress::MAX {
                    todo!("次のユニバースに進むか空いているところから取るかどちらがいいか?")
                } else {
                    adr.checked_add(1).unwrap().value() as i32
                }
            }
            None => DmxAddress::MIN.value() as i32,
        }
    });
}

/// "universe <number>"から<number>部分を取り出す。
fn parse_universe_id(universe_name: &str) -> UniverseId {
    // TODO: nameを自由に付けられるようにする
    let universe_id = universe_name
        .split(" ")
        .collect::<Vec<&str>>()
        .get(1)
        .expect(
            "cusstom universe name is not supproted at the moment: expected `universe <number>`",
        )
        .parse::<u8>()
        .expect(
            "custom universe name is not supproted at the moment: expected `universe <number>`",
        ); // TODO: エラーを返せるか？
    UniverseId::new(universe_id)
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;

    use crate::test_helpers;

    use super::*;
    use i_slint_backend_testing::{self as testing, ElementHandle};
    use tsukuyomi_core::doc::FakeFixtureDefRegistry;
    #[test]
    fn manufacturer_expands_after_click() {
        testing::init_no_event_loop();
        // TODO: DefRegistryのfakeを使う
        let mut app = App::new();
        setup(&mut app);

        let list_view: Vec<_> =
            ElementHandle::find_by_element_type_name(&app.ui, "FixtureListView").collect();
        assert_eq!(list_view.len(), 1);
        let list_view = &list_view[0];

        let manufacturer = list_view
            .visit_descendants(|el| {
                if el
                    .accessible_id()
                    .is_some_and(|id| id == "fixture-list-view-m-adb")
                {
                    ControlFlow::Break(el)
                } else {
                    ControlFlow::Continue(())
                }
            })
            .unwrap();
        assert!(!manufacturer.accessible_expanded().unwrap());
        manufacturer.invoke_accessible_expand_action();
        assert!(manufacturer.accessible_expanded().unwrap());
    }
}
