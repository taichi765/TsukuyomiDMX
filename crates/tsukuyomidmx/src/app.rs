use i_slint_backend_winit::WinitWindowAccessor;
use slint::{CloseRequestResponse, ComponentHandle, Model, Timer};
use std::{
    cell::OnceCell,
    collections::HashMap,
    error::Error,
    path::Path,
    rc::Rc,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
};
use tracing::{debug, instrument};
use tsukuyomidmx_core::{
    doc::{Doc, OutputPluginId},
    engine::{Engine, EngineCommand, EngineMessage},
    plugins::Plugin,
    prelude::{Fixture, FixtureDefId, UniverseId},
};

use crate::{
    models::{FixtureDefModel, FixtureModel, UniverseModel},
    tea::{fixture_list_view, preview_2d, universe_view},
    ui,
};

/// root struct
pub struct App {
    pub doc: Arc<Mutex<Doc>>,
    pub ui: ui::AppWindow,
    pub state: AppState,
    pub dispatcher: Dispatcher,
    pub shared_model_inner: SharedInnerModel,
    /// 永続化される状態だが、DocはPluginの詳細を知らないのでAppが保持する
    pub universe_configs: HashMap<UniverseId, UniverseConfig>,
    pub preview2d_timer: OnceCell<Timer>,

    // Engine
    pub engine_handle: OnceCell<thread::JoinHandle<()>>,
    pub command_tx: OnceCell<mpsc::Sender<EngineCommand>>,
    pub error_rx: OnceCell<mpsc::Receiver<EngineMessage>>,
}

impl App {
    pub fn new() -> Self {
        debug!("creating App instance");
        let doc = Arc::new(Mutex::new(
            Doc::try_new().expect("failed to initialize doc"),
        ));

        let ui = ui::AppWindow::new().unwrap();
        let dispatcher = Self::create_dispatcher();
        debug!("App instance created");
        Self {
            doc,
            ui,
            state: AppState {},
            dispatcher,
            shared_model_inner: SharedInnerModel {
                def_model: OnceCell::new(),
                fixture_model: OnceCell::new(),
                universe_model: OnceCell::new(),
            },
            universe_configs: HashMap::new(),
            preview2d_timer: OnceCell::new(),

            engine_handle: OnceCell::new(),
            command_tx: OnceCell::new(),
            error_rx: OnceCell::new(),
        }
    }

    pub fn from_dir(dir: &Path) -> Self {
        todo!()
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
        self.shared_model_inner
            .universe_model
            .set(UniverseModel::new(&mut self.doc.lock().unwrap()))
            .unwrap();

        fixture_list_view::setup(self);
        universe_view::setup(self);
        self.setup_engine();
        self.setup_window();
        preview_2d::setup(self);

        // TODO: ファイルから読み込む
        (0..2).for_each(|_| self.doc.lock().unwrap().add_universe());

        self.ui.run()?;
        self.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::Shutdown)
            .unwrap();

        self.engine_handle
            .take()
            .unwrap()
            .join()
            .expect("failed to finish Engine thread successfully");
        Ok(())
    }

    fn save(&self) {}

    /// Maximize, Toggle fullscreenなどの初期化
    #[instrument(skip_all)]
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

    #[instrument(skip_all)]
    fn setup_engine(&mut self) {
        let (command_tx, command_rx) = mpsc::channel::<EngineCommand>();
        let (error_tx, error_rx) = mpsc::channel::<EngineMessage>();

        let engine = Engine::new(self.doc.lock().unwrap().state_view(), command_rx, error_tx);

        let engine_handle = std::thread::Builder::new()
            .name("tsukuyomidmx-engine".into())
            .spawn(move || {
                debug!("starting engine loop");
                engine.start_loop()
            })
            .unwrap();
        self.engine_handle.set(engine_handle).unwrap();
        self.command_tx.set(command_tx).unwrap();
        self.error_rx.set(error_rx).unwrap();
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
    pub universe_model: OnceCell<Rc<UniverseModel>>,
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

#[derive(Debug)]
pub enum OutputPluginInfo {
    Artnet { target_ip: String },
    FTDI,
    Preview2D,
    Preview3D,
}

impl OutputPluginInfo {
    pub fn create_instance(&self) -> Box<dyn Plugin> {
        match self {
            Self::Artnet { target_ip } => todo!(),
            Self::FTDI => todo!(),
            Self::Preview2D => todo!(),
            Self::Preview3D => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct UniverseConfig {
    output_plugins: HashMap<OutputPluginId, OutputPluginInfo>,
}

impl UniverseConfig {
    pub fn new() -> Self {
        Self {
            output_plugins: HashMap::new(),
        }
    }

    pub fn output_plugins(&self) -> &HashMap<OutputPluginId, OutputPluginInfo> {
        &self.output_plugins
    }
}
