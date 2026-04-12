use anyhow::Context;
use i_slint_backend_winit::WinitWindowAccessor;
use serde::{Serialize, ser::SerializeSeq};
use slint::{CloseRequestResponse, ComponentHandle, Model, Timer};
use std::{
    cell::OnceCell,
    collections::HashMap,
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
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
    prelude::{Fixture, FixtureDefId, FixtureId, UniverseId},
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
    pub project_path: Option<PathBuf>,
    pub preview2d_timer: OnceCell<Timer>,

    // Engine
    pub engine_handle: OnceCell<thread::JoinHandle<()>>,
    pub command_tx: OnceCell<mpsc::Sender<EngineCommand>>,
    pub error_rx: OnceCell<mpsc::Receiver<EngineMessage>>,
}

impl App {
    pub fn new_empty() -> Self {
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
            project_path: None,
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

    /// [`project_path`][App::project_path]に保存する。
    fn save(&self) -> Result<(), anyhow::Error> {
        self.save_fixtures()?;
        self.save_functions()?;

        Ok(())
    }

    fn save_fixtures(&self) -> Result<(), anyhow::Error> {
        // TODO: dirtyフラグ
        let doc = self.doc.lock().unwrap().state_view();

        let fxt_file_path = self.get_project_path().join("fixtures.json");
        let mut fxt_file = File::create(&fxt_file_path)
            .with_context(|| format!("file {:?} does not exist", &fxt_file_path))?;
        doc.with_fixtures(|it| {
            let seq = FixturesSeq(it);
            serde_json::to_writer_pretty(&mut fxt_file, &seq).with_context(|| {
                format!("failed to serialize fixtures to file {:?}", fxt_file_path)
            })
        })?;
        fxt_file.flush().unwrap();
        Ok(())
    }

    fn save_functions(&self) -> Result<(), anyhow::Error> {
        // TODO: dirtyフラグ
        let doc = self.doc.lock().unwrap().state_view();

        let fun_dir = self.get_project_path().join("functions");
        doc.with_functions(|it| {
            for (_, fun) in it.iter() {
                let path = fun_dir.join(fun.id().to_string());
                let mut file = File::create(&path)
                    .with_context(|| format!("failed to create file {:?}", path))?;
                serde_json::to_writer(&file, &fun)
                    .with_context(|| format!("failed to serialize function {:?}", fun.id()))?;
                file.flush().unwrap();
            }
            Ok::<_, anyhow::Error>(())
        })?;

        let prototype_dir = self.get_project_path().join("function-prototypes");
        doc.with_function_prototypes(|it| {
            for (_, p) in it {
                let path = prototype_dir.join(p.id().to_string());
                let mut file = File::create(&path)
                    .with_context(|| format!("failed to create file {:?}", path))?;
                serde_json::to_writer_pretty(&file, &p).with_context(|| {
                    format!("failed to serialize function prototype {:?}", p.id())
                })?;
                file.flush().unwrap();
            }
            Ok(())
        })
    }

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
        Dispatcher(Rc::new(move |change| match change {}))
    }

    fn register_global_actions(self: &Arc<Self>) {
        let adopter = self.ui.global::<ui::MenuBarActions>();
        adopter.on_save({
            let weak = Arc::downgrade(self);
            move || {
                let action = Save;
                let app = weak.upgrade().unwrap();
                action.exec(&app).expect("failed to save");
            }
        });
    }

    fn register_key_bindings(self: &Arc<Self>) {
        self.ui.on_key_pressed({
            let weak = Arc::downgrade(self);

            move |ev| match ev.text.as_str() {
                "s" if ev.modifiers.control
                    && !ev.modifiers.alt
                    && !ev.modifiers.meta
                    && !ev.modifiers.shift =>
                {
                    weak.upgrade().unwrap().save().unwrap();
                    EventResult::Accept
                }
                _ => EventResult::Reject,
            }
        });
    }

    fn get_project_path(&self) -> PathBuf {
        self.project_path
            .lock()
            .unwrap()
            .get_or_insert_with(|| {
                rfd::FileDialog::new()
                    .set_can_create_directories(true)
                    .set_directory("/home/taichi765/Documents") // TODO: configから読み込む
                    .pick_folder()
                    .unwrap()
            })
            .clone()
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

/// Serialize `HashMap<FixtureId, Fixture>` without collecting items to `Vec`.
///
/// `HashMap<FixtureId, Fixture>` can't be serialized to JSON directory due to its key type.
/// Plus, Map doesn't make sense as JSON because id exists in Fixture so it's redundant.
/// However, collecting entire items to `Vec` is not memory-efficient,
/// so [`serialize_seq()`] is suitable here.
///
/// [`serialize_seq()`]:serde::Serializer::serialize_seq
struct FixturesSeq<'a>(&'a HashMap<FixtureId, Fixture>);

impl<'a> Serialize for FixturesSeq<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        for (_, fxt) in self.0.iter() {
            seq.serialize_element(fxt)?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use tsukuyomidmx_core::prelude::DmxAddress;

    use super::*;

    #[test]
    fn fixtures_seq_serializes_correctly() {
        let fixtures = vec![
            Fixture::new(
                "Fixture 1",
                UniverseId::MIN,
                DmxAddress::MIN,
                FixtureDefId::new_invalid(),
                "Mode 1",
                10.,
                20.,
            ),
            Fixture::new(
                "Fixture 2",
                UniverseId::MIN,
                DmxAddress::new(4).unwrap(),
                FixtureDefId::new_invalid(),
                "Mode 2",
                20.5,
                10.,
            ),
        ]
        .into_iter()
        .map(|fxt| (fxt.id(), fxt))
        .collect();

        let json = serde_json::to_string_pretty(&FixturesSeq(&fixtures)).unwrap();

        let deserialized: Vec<Fixture> = serde_json::from_str(&json).unwrap();
        let deserialized_map = deserialized
            .into_iter()
            .map(|fxt| (fxt.id(), fxt))
            .collect();

        assert_eq!(fixtures, deserialized_map);
    }
}
