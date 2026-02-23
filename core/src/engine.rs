use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::{info, trace, warn};
use uuid::Uuid;

use crate::doc::{DocStateView, OutputPluginId, ResolvedAddress};
use crate::fixture::{FixtureId, MergeMode};
use crate::functions::{FunctionCommand, FunctionId, FunctionRuntime};
use crate::plugins::{DmxFrame, Plugin};
use crate::universe::UniverseId;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

// TODO: unwrap, expectを減らす
/// Orchestrates [`FunctionRuntime`]s
pub struct Engine {
    doc: DocStateView,
    command_rx: Receiver<EngineCommand>,
    message_tx: Sender<EngineMessage>,

    active_runtimes: HashMap<FunctionId, Box<dyn FunctionRuntime>>,
    output_plugins: HashMap<OutputPluginId, Box<dyn Plugin>>,
    universe_states: HashMap<UniverseId, UniverseState>,
    output_map_cache: HashMap<OutputPluginId, Vec<UniverseId>>,
    live_values: HashMap<(FixtureId, String), u8>,

    should_shutdown: bool,
}

impl Engine {
    pub fn new(
        doc: DocStateView,
        command_rx: Receiver<EngineCommand>,
        message_tx: Sender<EngineMessage>,
    ) -> Self {
        Self {
            doc,
            command_rx,
            message_tx,
            active_runtimes: HashMap::new(),
            output_plugins: HashMap::new(),
            universe_states: HashMap::new(),
            output_map_cache: HashMap::new(),
            live_values: HashMap::new(),
            should_shutdown: false,
        }
    }

    pub fn start_loop(mut self) {
        info!("starting engine...");
        self.update_output_map_cache();
        loop {
            self.handle_engine_commands();

            self.universe_states.iter_mut().for_each(|(_, u)| u.clear());

            // apply live values before running function, so LTP channels will be overridden.
            let live_values = self.live_values.clone();
            live_values.iter().for_each(|((id, ch), v)| {
                self.write_universe(*id, ch, *v);
            });

            self.run_active_functions();

            self.dispatch_outputs();

            if self.should_shutdown {
                break;
            }
            std::thread::sleep(TICK_DURATION); //TODO: フレームレートを安定させる
        }
    }

    fn add_output_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.output_plugins.insert(plugin.id(), plugin);
        trace!("added output plugin");
    }

    fn handle_engine_commands(&mut self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            trace!(?cmd, "received command");
            match cmd {
                EngineCommand::StartFunction(id) => self.start_function(id),
                EngineCommand::StopFunction(id) => self.stop_function(id),
                EngineCommand::AddPlugin(p) => self.add_output_plugin(p),
                EngineCommand::UniverseAdded(id) => {
                    if let None = self.universe_states.insert(id, UniverseState::new()) {
                        warn!(
                            "UniverseAdded: universe id {id:?} already exists in Engine::universes"
                        );
                    }
                }
                EngineCommand::UniverseRemoved(id) => {
                    if let None = self.universe_states.remove(&id) {
                        warn!(
                            "UniverseRemoved: universe id {id:?} does not exists in Engine::universes"
                        );
                    }
                }
                EngineCommand::SetLiveValue {
                    fixture_id,
                    channel,
                    value,
                } => {
                    if value == 0 {
                        // エントリが存在しなかった場合も何もしない
                        let _ = self.live_values.remove(&(fixture_id, channel));
                    } else {
                        let _ = self.live_values.insert((fixture_id, channel), value);
                    }
                }
                EngineCommand::OutputMapChanged => self.update_output_map_cache(),
                EngineCommand::Shutdown => self.should_shutdown = true,
            }
        }
    }

    fn run_active_functions(&mut self) {
        let mut commands_list = Vec::new();
        {
            for (function_id, runtime) in &mut self.active_runtimes {
                self.doc.with_functions(|it| {
                    let data = it.get(function_id).expect("todo: これはsafe?");
                    commands_list.append(&mut runtime.run(data, TICK_DURATION));
                });
            }
        }

        for command in commands_list {
            match command {
                FunctionCommand::StartFunction(function_id) => self.start_function(function_id),
                FunctionCommand::StopFuntion(function_id) => self.stop_function(function_id),
                FunctionCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => self.write_universe(fixture_id, &channel, value),
                FunctionCommand::StartFade {
                    from_id,
                    to_id,
                    chaser_id,
                    duration,
                } => self.start_fade(from_id, to_id, chaser_id, duration),
            }
        }
    }

    fn dispatch_outputs(&mut self) {
        self.output_map_cache.par_iter().for_each(|(p_id, u_ids)| {
            let Some(plugin) = self.output_plugins.get(p_id) else {
                warn!(plugin_id = %p_id, "plugin not found"); // FIXME: message_txでエラーを送るべき？
                return;
            };
            u_ids.iter().for_each(|u_id| {
                let Some(universe_data) = self.universe_states.get(u_id) else {
                    warn!(universe_id = ?u_id, "universe state not created");
                    return;
                };
                if let Err(e) = plugin.send_dmx(*u_id, DmxFrame::from(universe_data.values)) {
                    self.message_tx
                        .send(EngineMessage::ErrorOccured(EngineError {
                            context: ErrorContext::SendingDmx {
                                universe_id: *u_id,
                                plugin_id: *p_id,
                            },
                            source: Box::new(e),
                        }))
                        .unwrap();
                }
            });
        });
    }

    ///既にstartしてた場合は何もしない
    fn start_function(&mut self, function_id: FunctionId) {
        let runtime = self.doc.with_functions(|it| {
            it.get(&function_id)
                .expect("todo: エラー返す")
                .create_runtime()
        });
        self.active_runtimes.insert(function_id, runtime);
    }

    ///既にstopしてた/そもそも存在しなかった場合、何もしない
    fn stop_function(&mut self, function_id: FunctionId) {
        self.active_runtimes.remove(&function_id);
    }

    fn write_universe(&mut self, _fixture_id: FixtureId, _channel: &str, _value: u8) {
        todo!("resolve_addressをどっかから取ってくる")
        /*match self.doc.read().resolve_address(fixture_id, channel) {
            Ok((universe_id, address)) => {
                let universe = self
                    .universe_states
                    .get_mut(&universe_id)
                    .expect(format!("universe states not found: {:?}", universe_id).as_str()); // FIXME: EngineMessageで通知する
                universe.set_value(address, value);
            }
            Err(e) => {
                if let Err(send_err) =
                    self.message_tx
                        .send(EngineMessage::ErrorOccured(EngineError {
                            context: ErrorContext::ResolvingAddress {
                                fixture_id,
                                channel: String::from(channel),
                            },
                            source: Box::new(e),
                        }))
                {
                    error!("engine: failed to send error: {}", send_err);
                }
            }
        }*/
    }

    fn start_fade(&mut self, _from_id: Uuid, _to_id: Uuid, _chaser_id: Uuid, _duration: Duration) {
        //必要な値だけを取り出す
        /*let (from_values, to_values) = {
            let from_scene = self.get_function(from_id).as_ref();
            let from_scene = match from_scene.function_type() {
                FunctionType::Scene => (from_scene as &dyn Any)
                    .downcast_ref::<StaticSceneData>()
                    .unwrap(),
                _ => panic!("unimplemented type"),
            };

            let to_scene = self.get_function(to_id).as_ref();
            let to_scene = match to_scene.function_type() {
                FunctionType::Scene => (to_scene as &dyn Any)
                    .downcast_ref::<StaticSceneData>()
                    .unwrap(),
                _ => panic!("unimplemented type"),
            };
            // TODO: 無駄なclone?
            (from_scene.values().clone(), to_scene.values().clone())
        };
        let fader = Fader::new(
            self.next_internal_function_id(),
            to_id,
            chaser_id,
            from_values,
            to_values,
            duration,
        );
        let fader_id = fader.id();
        self.push_function(Box::new(fader))
            .expect("functionの追加に失敗しました");
        self.start_function(fader_id);*/
    }

    // FIXME: universe_settingsごとプッシュ型の方がいいか？
    fn update_output_map_cache(&mut self) {
        todo!()
        /*trace!("updating output map cache");
        let doc = self.doc.read();
        let mut new_map: HashMap<OutputPluginId, Vec<UniverseId>> = HashMap::new();
        for (u_id, setting) in doc.universe_settings() {
            trace!("plugin in doc({u_id:?}):{:?}", setting.output_plugins());
            setting.output_plugins().iter().for_each(|p_id| {
                if let Some(universes) = new_map.get_mut(p_id) {
                    universes.push(*u_id);
                } else {
                    new_map.insert(*p_id, vec![*u_id]);
                }
            });
        }

        self.output_map_cache = new_map;*/
    }
}

/// Message from the main thread to [`Engine`]
#[derive(Debug)]
pub enum EngineCommand {
    // Commands
    StartFunction(FunctionId),
    StopFunction(FunctionId),
    AddPlugin(Box<dyn Plugin>),
    SetLiveValue {
        fixture_id: FixtureId,
        channel: String,
        value: u8,
    },
    Shutdown,

    // Events
    OutputMapChanged, // FIXME: Docの監視にメインスレッドを介すのは正しいか？
    UniverseAdded(UniverseId),
    UniverseRemoved(UniverseId),
}

/// Message from [`Engine`] to the main thread
pub enum EngineMessage {
    ErrorOccured(EngineError),
}

// TODO: thiserror使う
/// The errors occured in [`Engine`]
#[derive(Debug)]
pub struct EngineError {
    context: ErrorContext,
    source: Box<dyn Error + Send + Sync>,
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.context, self.source)
    }
}

impl Error for EngineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub enum ErrorContext {
    ResolvingAddress {
        fixture_id: FixtureId,
        channel: String,
    },
    RunningFunction {
        function_id: FunctionId,
    },
    SendingDmx {
        universe_id: UniverseId,
        plugin_id: OutputPluginId,
    },
}

pub(crate) struct UniverseState {
    values: [u8; 512],
}

impl UniverseState {
    pub fn new() -> Self {
        Self { values: [0; 512] }
    }

    pub(crate) fn clear(&mut self) {
        self.values.fill(0);
    }

    pub(crate) fn set_value(&mut self, resolved_address: ResolvedAddress, value: u8) {
        let idx = resolved_address.address.value() - 1; // address -> index conversion
        match resolved_address.merge_mode {
            MergeMode::HTP => {
                if value > self.values[idx] {
                    self.values[idx] = value
                }
            }
            MergeMode::LTP => self.values[idx] = value,
        }
    }
}

#[cfg(test)]
mod tests {}
