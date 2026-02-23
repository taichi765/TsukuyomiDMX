use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::{ComponentHandle, MapModel, Model, ToSharedString, VecModel};
use tsukuyomi_core::{
    doc::{Doc, DocStateView, FixtureAddError, FixtureDefNotFound, ModeNotFound, ValidateError},
    prelude::{DmxAddress, Fixture, FixtureDefId, UniverseId},
};
use uuid::Uuid;

use crate::{
    app::{App, AppAction, Dispatcher},
    models::FixtureDefModel,
    ui,
};

pub fn setup(app: &mut App) {
    let doc_view = app.doc.lock().unwrap().state_view();
    let doc_clone = Arc::clone(&app.doc);
    let adopter = app.ui.global::<ui::FixtureListStore>();
    let ui_handle = app.ui.as_weak();

    let def_model = FixtureDefModel::create(&mut app.doc.lock().unwrap());
    app.shared_model_inner
        .def_model
        .set(Rc::clone(&def_model))
        .unwrap();
    let map_model = MapModel::new(def_model, |(manufacturer, defs)| ui::ManufacturerModel {
        expanded: false,
        fixtures: Rc::new(VecModel::from(
            defs.iter()
                .map(|(id, model)| ui::FixtureModel {
                    id: id.to_shared_string(),
                    modes: Rc::new(VecModel::from(vec!["Mode 1".into()])).into(), // TODO: metadataから取得したい,
                    name: model.to_owned(),
                })
                .collect::<Vec<_>>(),
        ))
        .into(),
        manufacturer: manufacturer,
    });

    adopter.set_model(Rc::new(map_model).into());

    adopter.on_patch(move |universe, address, fixture_def_id, mode, pos| {
        let universe_id = parse_universe_id(universe.as_str());
        let def_id = FixtureDefId::try_from(fixture_def_id.as_str()).unwrap();
        let default_fxt_name = {
            let model_name =
                doc_view.with_fixture_defs(|it| it.get(&def_id).expect("todo").model().to_string());
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
                    FixtureAddError::FixtureDefNotFound(FixtureDefNotFound {
                        fixture_id: _,
                        fixture_def_id: def_id,
                        source: e,
                    }) => format!("couldn't find fixture definition {}: {:?}", def_id, e)
                        .to_shared_string(),
                    FixtureAddError::ModeNotFound(ModeNotFound {
                        fixture_def: def_id,
                        mode,
                    }) => format!("there's no mode {mode} in the definition {def_id:?}")
                        .to_shared_string(),
                    FixtureAddError::AddressValidateError(ValidateError::AddressConflicted(
                        conflicts,
                    )) => {
                        format!("{} addresses conflicted: {:?}", conflicts.len(), conflicts)
                            .to_shared_string() // TODO: 見やすくする
                    }
                    FixtureAddError::FixtureAlreadyExists(fxt_id) => {
                        format!("fixture id {fxt_id:?} already exists").to_shared_string()
                    }
                }
            }
        }
    });

    adopter.on_get_modes(move |def_id| {
        let fixture_model = ui_handle
            .unwrap()
            .global::<ui::FixtureListStore>()
            .get_model()
            .iter()
            .find_map(|m| m.fixtures.iter().find(|fxt| fxt.id == def_id))
            .unwrap();
        fixture_model.modes
    });

    adopter.on_get_next_address(|universe| {
        let uni_id = parse_universe_id(&universe);
        todo!()
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
