use slint::{ComponentHandle, Model};
use std::{
    cell::OnceCell,
    error::Error,
    rc::Rc,
    sync::{Arc, Mutex},
};
use tsukuyomi_core::{
    doc::Doc,
    prelude::{Fixture, FixtureDefId},
};

use crate::{
    models::{FixtureDefModel, FixtureModelInner},
    tea::fixture_list_view,
    ui::{self, AppWindow},
};

pub struct App {
    pub doc: Arc<Mutex<Doc>>,
    pub ui: ui::AppWindow,
    pub state: AppState,
    pub dispatcher: Dispatcher,
    pub local_models: LocalModels,
    pub shared_model_inner: SharedInnerModel,
}

impl App {
    pub fn new() -> Self {
        let doc = Arc::new(Mutex::new(Doc::new()));
        let ui = AppWindow::new().unwrap();
        let dispatcher = Self::create_dispatcher();
        Self {
            doc,
            ui,
            state: AppState {},
            dispatcher,
            local_models: LocalModels {},
            shared_model_inner: SharedInnerModel {
                def_model: OnceCell::new(),
                fixture_model: OnceCell::new(),
            },
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        fixture_list_view::setup(self);
        self.ui.run()?;
        Ok(())
    }

    fn create_dispatcher() -> Dispatcher {
        Dispatcher(Rc::new(move |action| match action {}))
    }
}

pub struct AppState {}

pub struct LocalModels {}

/// これらのModelにMapModel等を使ってuiに渡す、共通化部分
pub struct SharedInnerModel {
    pub def_model: OnceCell<Rc<FixtureDefModel>>,
    pub fixture_model: OnceCell<Rc<FixtureModelInner>>,
}

pub enum AppAction {}

pub struct Dispatcher(Rc<dyn FnMut(AppAction)>);

impl Clone for Dispatcher {
    /// Cheap clone.
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Dispatcher {
    pub fn dispatch(&self, action: AppAction) {
        (self.0)(action)
    }
}
