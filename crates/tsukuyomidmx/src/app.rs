use anyhow::Context;
use i_slint_backend_winit::WinitWindowAccessor;
use i_slint_core::{input::KeyEventType, items::EventResult};
use serde::{Deserialize, Serialize, ser::SerializeSeq};
use slint::{
    CloseRequestResponse, ComponentHandle, Model, Timer, ToSharedString,
    language::KeyboardModifiers,
};
use std::{
    cell::OnceCell,
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock, Weak,
        mpsc::{self, Sender},
    },
    thread,
};
use tracing::{debug, instrument};
use tsukuyomidmx_core::{
    doc::{Doc, OutputPluginId},
    effects::{
        Effect, EffectChange, EffectSpec, EffectSpecId, EffectTemplate, EffectTemplateId,
        FixtureQuery, SimpleEffectBody,
    },
    engine::{Engine, EngineCommand, EngineMessage},
    plugins::{OlaPlugin, Plugin},
    prelude::{DmxAddress, EffectId, Fixture, FixtureDefId, FixtureId, UniverseId},
};

use crate::{
    Observable,
    models::{EffectEditorData, EffectEditorModel, FixtureDefModel, FixtureModel, UniverseModel},
    ui,
    ui_handlers::{effect_editor, effect_tree_view, fixture_list_view, preview_2d, universe_view},
};

/// root struct
pub struct App {
    pub doc: Arc<Mutex<Doc>>,
    pub ui: ui::AppWindow,
    pub state: Arc<RwLock<AppState>>,
    pub dispatcher: Dispatcher,
    pub shared_model: SharedInnerModel,
    /// 永続化される状態だが、DocはPluginの詳細を知らないのでAppが保持する
    pub universe_configs: HashMap<UniverseId, UniverseConfig>,
    project_path: Mutex<Option<PathBuf>>,
    pub keymap: HashMap<(), Box<dyn Action>>,
    pub preview2d_timer: OnceCell<Timer>,

    // Engine
    engine_handle: Mutex<OnceCell<thread::JoinHandle<()>>>,
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
        let state = Arc::new(RwLock::new(AppState::new()));
        debug!("App instance created");
        Self {
            doc,
            ui,
            state: Arc::clone(&state),
            dispatcher: create_dispatcher(state),
            shared_model: SharedInnerModel {
                def_model: OnceCell::new(),
                fixture_model: OnceCell::new(),
                universe_model: OnceCell::new(),
            },
            universe_configs: HashMap::new(),
            project_path: Mutex::new(None),
            keymap: HashMap::new(),
            preview2d_timer: OnceCell::new(),

            engine_handle: Mutex::new(OnceCell::new()),
            command_tx: OnceCell::new(),
            error_rx: OnceCell::new(),
        }
    }

    pub fn from_dir(dir: &Path) -> Result<Self, anyhow::Error> {
        debug!("creating App instance");
        let fxt_file_path = dir.join("fixtures.json");
        let fxt_file = File::open(&fxt_file_path)
            .with_context(|| format!("failed to open file {:?}", fxt_file_path))?;
        // OPTIM: ストリーミングでVecを介さずにHashMapに変換できると嬉しいかも
        let fixtures: Vec<Fixture> = serde_json::from_reader(&fxt_file)
            .with_context(|| format!("failed to deserialize fixtures from {:?}", fxt_file_path))?;
        let fixtures = fixtures.into_iter().map(|fxt| (fxt.id(), fxt)).collect();

        let effects_dir = dir.join("effects");
        let effects =
            std::fs::read_dir(effects_dir)?.try_fold(HashMap::new(), |mut acc, entry| {
                let entry = entry?;
                let file = File::open(entry.path())
                    .with_context(|| format!("failed to open file {:?}", entry.path()))?;
                let effect: Effect = serde_json::from_reader(&file).with_context(|| {
                    format!("failed to deserialize function from {:?}", entry.path())
                })?;
                acc.insert(effect.id(), effect);
                anyhow::Ok(acc)
            })?;

        let specs_dir = dir.join("effect-specs");
        let specs = std::fs::read_dir(specs_dir)?.try_fold(HashMap::new(), |mut acc, entry| {
            let entry = entry?;
            let file = File::open(entry.path())
                .with_context(|| format!("failed to open file {:?}", entry.path()))?;
            let spec: EffectSpec = serde_json::from_reader(&file).with_context(|| {
                format!(
                    "failed to deserialize function prototype from {:?}",
                    entry.path()
                )
            })?;
            acc.insert(spec.id(), spec);
            anyhow::Ok(acc)
        })?;

        let tmpls_dir = dir.join("effect-templates");
        let tmpls = std::fs::read_dir(tmpls_dir)?.try_fold(HashMap::new(), |mut acc, entry| {
            let entry = entry?;
            let file = File::open(entry.path())
                .with_context(|| format!("failed to open file {:?}", entry.path()))?;
            let tmpl: EffectTemplate = serde_json::from_reader(&file).with_context(|| {
                format!(
                    "failed to deserialize function prototype from {:?}",
                    entry.path()
                )
            })?;
            acc.insert(tmpl.id(), tmpl);
            anyhow::Ok(acc)
        })?;

        let universes_path = dir.join("universes.json");
        let universes_file = File::open(&universes_path)
            .with_context(|| format!("failed to open file {:?}", universes_path))?;
        let univ_dto: UniverseConfigListDto = serde_json::from_reader(&universes_file)
            .with_context(|| {
                format!("failed to deserialize universes from {:?}", universes_path)
            })?;
        let universes = univ_dto.data.iter().map(|(u_id, _)| *u_id).collect();
        let universe_cfgs: HashMap<UniverseId, UniverseConfig> = univ_dto.into();

        let doc = Arc::new(Mutex::new(
            Doc::from_existing_data(fixtures, specs, tmpls, effects, universes)
                .with_context(|| format!("failed to create Doc"))?,
        ));

        let ui = ui::AppWindow::new().unwrap();
        ui.set_project_path(dir.to_str().unwrap().to_shared_string());
        let state = Arc::new(RwLock::new(AppState::new()));

        Ok(Self {
            doc: Arc::clone(&doc),
            ui,
            state: Arc::clone(&state),
            dispatcher: create_dispatcher(state),
            shared_model: SharedInnerModel {
                fixture_model: OnceCell::new(),
                def_model: OnceCell::new(),
                universe_model: OnceCell::new(),
            },
            universe_configs: universe_cfgs,
            project_path: Mutex::new(Some(dir.to_path_buf())),
            keymap: HashMap::new(),
            preview2d_timer: OnceCell::new(),
            engine_handle: Mutex::new(OnceCell::new()),
            command_tx: OnceCell::new(),
            error_rx: OnceCell::new(),
        })
    }

    pub fn run(self: Arc<Self>) -> Result<(), Box<dyn Error>> {
        self.shared_model
            .def_model
            .set(FixtureDefModel::create(&mut self.doc.lock().unwrap()))
            .unwrap();
        self.shared_model
            .fixture_model
            .set(FixtureModel::create(&mut self.doc.lock().unwrap()))
            .unwrap();
        self.shared_model
            .universe_model
            .set(UniverseModel::new(&mut self.doc.lock().unwrap()))
            .unwrap();

        fixture_list_view::setup(&self);
        universe_view::setup(&self);
        effect_tree_view::setup(&self);
        self.setup_engine();
        self.setup_window();
        preview_2d::setup(&self);
        effect_editor::setup(&self);

        // TODO: この部分はuniverses.jsonから読み込んでやる
        let plugin = Box::new(OlaPlugin::new().unwrap());
        let p_id = plugin.id();
        self.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::AddPlugin(plugin))
            .unwrap();
        self.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::AddPluginDestination {
                plugin: p_id,
                dest_universe: UniverseId::new(0),
            })
            .unwrap();
        self.register_global_actions();
        self.register_key_bindings();

        // TODO: ファイルから読み込む
        (0..2).for_each(|_| self.doc.lock().unwrap().add_universe());
        self.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::UniverseAdded(UniverseId::new(0)))
            .unwrap();

        self.ui.run()?;

        // 後片付け
        self.command_tx
            .get()
            .unwrap()
            .send(EngineCommand::Shutdown)
            .unwrap();

        self.engine_handle
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .join()
            .expect("failed to finish Engine thread successfully");
        Ok(())
    }

    /// [`project_path`][App::project_path]に保存する。
    fn save(&self) -> Result<(), anyhow::Error> {
        self.save_fixtures()?;
        self.save_effects()?;
        self.seva_universes()?;

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

    fn save_effects(&self) -> Result<(), anyhow::Error> {
        // TODO: dirtyフラグ
        let doc = self.doc.lock().unwrap().state_view();

        let effect_dir = self.get_project_path().join("effects");
        doc.with_effects(|it| {
            for (_, effect) in it.iter() {
                let path = effect_dir.join(format!("{}__{}.json", effect.id(), effect.name()));
                let mut file = File::create(&path)
                    .with_context(|| format!("failed to create file {:?}", path))?;
                serde_json::to_writer_pretty(&file, &effect)
                    .with_context(|| format!("failed to serialize function {:?}", effect.id()))?;
                file.flush().unwrap();
            }
            Ok::<_, anyhow::Error>(())
        })?;

        let spec_dir = self.get_project_path().join("effect-specs");
        doc.with_effect_specs(|it| {
            for (_, spec) in it {
                let path = spec_dir.join(format!("{}__{}.json", spec.id(), spec.name()));
                let mut file = File::create(&path)
                    .with_context(|| format!("failed to create file {:?}", path))?;
                serde_json::to_writer_pretty(&file, &spec).with_context(|| {
                    format!("failed to serialize function prototype {:?}", spec.id())
                })?;
                file.flush().unwrap();
            }
            anyhow::Ok(())
        })?;

        let tmpl_dir = self.get_project_path().join("effect-templates");
        doc.with_effect_templates(|it| {
            for (_, tmpl) in it {
                let path = tmpl_dir.join(format!("{}__{}.json", tmpl.name(), tmpl.id()));
                let mut file = File::create(&path)
                    .with_context(|| format!("failed to create file {:?}", path))?;
                serde_json::to_writer_pretty(&file, &tmpl).with_context(|| {
                    format!(
                        "failed to serialize effect template {}({:?})",
                        tmpl.name(),
                        tmpl.id()
                    )
                })?;
                file.flush().unwrap();
            }
            anyhow::Ok(())
        })
    }

    fn seva_universes(&self) -> Result<(), anyhow::Error> {
        let path = self.get_project_path().join("universes.json");
        let mut file = File::create(path)?;
        let dto = UniverseConfigListDto::from(self.universe_configs.clone());
        serde_json::to_writer(&file, &dto)?;
        file.flush().unwrap();
        Ok(())
    }

    /// Maximize, Toggle fullscreenなどの初期化
    #[instrument(skip_all)]
    fn setup_window(&self) {
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
    fn setup_engine(&self) {
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
        self.engine_handle
            .lock()
            .unwrap()
            .set(engine_handle)
            .unwrap();
        self.command_tx.set(command_tx).unwrap();
        self.error_rx.set(error_rx).unwrap();
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

pub struct AppState {
    current_effect_id: Observable<Option<AnyEffectId>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            current_effect_id: Observable::new(None),
        }
    }

    pub fn current_effect_id(&self) -> Observable<Option<AnyEffectId>> {
        self.current_effect_id.clone()
    }
}

/// これらのModelにMapModel等を使ってuiに渡す、共通化部分
pub struct SharedInnerModel {
    pub def_model: OnceCell<Rc<FixtureDefModel>>,
    pub fixture_model: OnceCell<Rc<FixtureModel>>,
    pub universe_model: OnceCell<Rc<UniverseModel>>,
}

/// [`AppState`]を変更するコマンド
pub enum AppStateChange {
    SetSelectedFunction(AnyEffectId),
}

pub struct Dispatcher(Rc<dyn Fn(AppStateChange)>);

impl Clone for Dispatcher {
    /// Cheap clone.
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Dispatcher {
    pub fn dispatch(&self, change: AppStateChange) {
        (self.0)(change)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
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

#[derive(Serialize, Deserialize)]
struct UniverseConfigListDto {
    data: Vec<(UniverseId, Vec<(OutputPluginId, OutputPluginInfo)>)>,
}

impl From<HashMap<UniverseId, UniverseConfig>> for UniverseConfigListDto {
    fn from(value: HashMap<UniverseId, UniverseConfig>) -> Self {
        Self {
            data: value
                .into_iter()
                .map(|(u_id, cfg)| (u_id, cfg.output_plugins.into_iter().collect()))
                .collect(),
        }
    }
}

impl Into<HashMap<UniverseId, UniverseConfig>> for UniverseConfigListDto {
    fn into(self) -> HashMap<UniverseId, UniverseConfig> {
        self.data
            .into_iter()
            .map(|(u_id, plugins)| {
                (
                    u_id,
                    UniverseConfig {
                        output_plugins: plugins.into_iter().collect(),
                    },
                )
            })
            .collect()
    }
}

pub trait Action {
    fn exec(&self, app: &App) -> Result<(), anyhow::Error>;
}

struct Save;

impl Action for Save {
    fn exec(&self, app: &App) -> Result<(), anyhow::Error> {
        app.save()
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

// TODO: core側で定義したほうがいいかも
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnyEffectId {
    Effect(EffectId),
    Spec(EffectSpecId),
    Template(EffectTemplateId),
}

impl AnyEffectId {
    pub fn unwrap_effect(&self) -> EffectId {
        if let Self::Effect(id) = self {
            *id
        } else {
            panic!("EffectLikeId::unwrap_effect() is called on {:?}", self);
        }
    }

    pub fn unwrap_spec(&self) -> EffectSpecId {
        if let Self::Spec(id) = self {
            *id
        } else {
            panic!("EffectLikeId::unwrap_spec() is called on {:?}", self);
        }
    }

    pub fn unwrap_template(&self) -> EffectTemplateId {
        if let Self::Template(id) = self {
            *id
        } else {
            panic!("EffectLikeId::unwrap_template() is called on {:?}", self);
        }
    }
}

fn create_dispatcher(state: Arc<RwLock<AppState>>) -> Dispatcher {
    Dispatcher(Rc::new(move |change| match change {
        AppStateChange::SetSelectedFunction(id) => {
            state.write().unwrap().current_effect_id.set(Some(id));
        }
    }))
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
