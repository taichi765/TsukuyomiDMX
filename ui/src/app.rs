use i_slint_backend_winit::WinitWindowAccessor;
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
    models::{FixtureDefModel, FixtureModel},
    tea::{fixture_list_view, universe_view},
    ui,
};

/// root struct
pub struct App {
    pub doc: Arc<Mutex<Doc>>,
    pub ui: ui::AppWindow,
    pub state: AppState,
    pub dispatcher: Dispatcher,
    pub shared_model_inner: SharedInnerModel,
}

impl App {
    pub fn new() -> Self {
        let doc = Arc::new(Mutex::new(
            Doc::try_new().expect("failed to initialize doc"),
        ));

        let ui = ui::AppWindow::new().unwrap();
        let dispatcher = Self::create_dispatcher();
        Self {
            doc,
            ui,
            state: AppState {},
            dispatcher,
            shared_model_inner: SharedInnerModel {
                def_model: OnceCell::new(),
                fixture_model: OnceCell::new(),
            },
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.shared_model_inner
            .def_model
            .set(FixtureDefModel::create(&mut self.doc.lock().unwrap()))
            .unwrap();
        self.shared_model_inner
            .fixture_model
            .set(FixtureModel::create(&mut self.doc.lock().unwrap()))
            .unwrap();

        fixture_list_view::setup(self);
        universe_view::setup(self);
        self.setup_window();

        self.ui.run()?;
        Ok(())
    }

    /// Maximize, Toggle fullscreenなどの初期化
    fn setup_window(&mut self) {
        self.ui.on_start_drag({
            let ui_handle = self.ui.as_weak();

            move || {
                let ui = ui_handle.unwrap();
                ui.window().with_winit_window(|w| w.drag_window());
            }
        });

        self.ui.on_minimize({
            let ui_handle = self.ui.as_weak();

            move || {
                let ui = ui_handle.unwrap();
                ui.window().set_minimized(true);
            }
        });

        // TODO: toggle-fullscreen

        self.ui.on_close({
            let ui_handle = self.ui.as_weak();

            move || {
                let ui = ui_handle.unwrap();
                ui.window().hide().unwrap()
            }
        });
    }

    fn create_dispatcher() -> Dispatcher {
        Dispatcher(Rc::new(move |action| match action {}))
    }
}

pub struct AppState {}

/// これらのModelにMapModel等を使ってuiに渡す、共通化部分
pub struct SharedInnerModel {
    pub def_model: OnceCell<Rc<FixtureDefModel>>,
    pub fixture_model: OnceCell<Rc<FixtureModel>>,
}

pub enum AppAction {}

pub struct Dispatcher(Rc<dyn Fn(AppAction)>);

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
