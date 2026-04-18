use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::doc::{DocStateView, OutputPluginId, ResolveError, ResolvedAddress};
use crate::effects::{EffectCommand, EffectId, EffectRuntime};
use crate::fixture::{FixtureId, MergeMode};
use crate::plugins::{DmxFrame, Plugin};
use crate::universe::UniverseId;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

//TODO: なんとなくpubにしているものがある pub(crate)とかも活用したい

const TICK_DURATION: Duration = Duration::from_millis(100);

// TODO: unwrap, expectを減らす
/// Functionを実行する
pub struct Engine {
    doc: DocStateView,
    command_rx: Receiver<EngineCommand>,
    message_tx: Sender<EngineMessage>,

    active_runtime: Option<Box<dyn EffectRuntime>>,
    /// Pluginインスタンス
    output_plugins: HashMap<OutputPluginId, Box<dyn Plugin>>,
    /// どのPluginがどのUniverseに出力するか
    output_map: HashMap<OutputPluginId, HashSet<UniverseId>>,

    universe_states: HashMap<UniverseId, UniverseState>,
    /// シンプル卓など一時的な値
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
            active_runtime: None,
            output_plugins: HashMap::new(),
            universe_states: HashMap::new(),
            output_map: HashMap::new(),
            live_values: HashMap::new(),
            should_shutdown: false,
        }
    }

    pub fn start_loop(mut self) {
        info!("starting engine...");
        loop {
            self.handle_engine_commands();

            if self.should_shutdown {
                break;
            }

            self.universe_states.iter_mut().for_each(|(_, u)| u.clear());

            // apply live values before running function, so LTP channels will be overridden.
            let live_values = self.live_values.clone();
            live_values.iter().for_each(|((id, ch), v)| {
                self.write_universe_with_channel_name(*id, ch, *v);
            });

            self.run_active_function();

            self.dispatch_outputs();

            std::thread::sleep(TICK_DURATION); //TODO: フレームレートを安定させる
        }
    }

    fn handle_engine_commands(&mut self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            debug!(?cmd, "received command");
            match cmd {
                EngineCommand::StartFunction(id) => self.start_function(id),
                EngineCommand::StopFunction => self.stop_function(),
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
                EngineCommand::AddPlugin(p) => {
                    let id = p.id();
                    self.output_plugins.insert(id, p);
                    self.output_map.insert(id, HashSet::new());
                }
                EngineCommand::AddPluginDestination {
                    plugin,
                    dest_universe,
                } => {
                    if !self.output_plugins.contains_key(&plugin) {
                        self.message_tx
                            .send(EngineMessage::ErrorOccured(
                                EngineError::OutputPluginNotFound { id: plugin },
                            ))
                            .expect("failed to send message from engine")
                    }
                    let dests = self.output_map.get_mut(&plugin).unwrap();
                    if dests.contains(&dest_universe) {
                        warn!(
                            ?plugin,
                            ?dest_universe,
                            "universe already exists in output_map. command is ignored."
                        )
                    } else {
                        dests.insert(dest_universe);
                    }
                }
                EngineCommand::RemovePluginDestination {
                    plugin,
                    dest_universe,
                } => {
                    if !self.output_plugins.contains_key(&plugin) {
                        self.message_tx
                            .send(EngineMessage::ErrorOccured(
                                EngineError::OutputPluginNotFound { id: plugin },
                            ))
                            .expect("failed to send message from engine");
                    }
                    let dests = self.output_map.get_mut(&plugin).unwrap();
                    if !dests.contains(&dest_universe) {
                        warn!(
                            ?plugin,
                            ?dest_universe,
                            "universe does not exist in output_map. command is ignored."
                        );
                    } else {
                        dests.insert(dest_universe);
                    }
                }
                EngineCommand::Shutdown => self.should_shutdown = true,
            }
        }
    }

    fn run_active_function(&mut self) {
        let Some(commands) = self
            .active_runtime
            .as_mut()
            .map(|rt| rt.run(TICK_DURATION, self.doc.clone()))
        else {
            return;
        };

        for command in commands {
            match command {
                EffectCommand::StartEffect(function_id) => self.start_function(function_id),
                EffectCommand::StopEffect => self.stop_function(),
                EffectCommand::WriteUniverse {
                    fixture_id,
                    channel,
                    value,
                } => self.write_universe(fixture_id, channel, value),
            }
        }
    }

    fn dispatch_outputs(&mut self) {
        self.output_map.par_iter().for_each(|(p_id, u_ids)| {
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
                        .send(EngineMessage::ErrorOccured(EngineError::SendingDmx {
                            universe_id: *u_id,
                            plugin_id: *p_id,
                            source: Box::new(e),
                        }))
                        .unwrap();
                }
            });
        });
    }

    /// 既にactiveなfunctionがあった場合上書きされる
    fn start_function(&mut self, function_id: EffectId) {
        let res = self.doc.with_effects(|it| {
            it.get(&function_id)
                .map(|fx| fx.create_runtime(self.doc.clone()))
                .ok_or(EngineError::FunctionNotFound { function_id })
        });

        match res {
            Ok(rt) => self.active_runtime = Some(rt),
            Err(e) => self
                .message_tx
                .send(EngineMessage::ErrorOccured(e))
                .expect("failed to send message from engine thread"),
        }
    }

    /// 現在activeなfuncitonを止める。
    ///
    /// activeなfunctionが無かった場合何もしない。
    fn stop_function(&mut self) {
        self.active_runtime = None;
    }

    fn write_universe(&mut self, fxt_id: FixtureId, channel: usize, value: u8) {
        match self.doc.resolve_address_with_offset(fxt_id, channel) {
            Ok(resolved_address) => {
                let univ = self
                    .universe_states
                    .get_mut(&resolved_address.universe)
                    .expect("todo");
                univ.set_value(resolved_address, value);
            }
            Err(e) => self
                .message_tx
                .send(EngineMessage::ErrorOccured(EngineError::ResolveAddress(e)))
                .expect("failed to send message from engine thread"),
        }
    }

    fn write_universe_with_channel_name(
        &mut self,
        fixture_id: FixtureId,
        channel: &str,
        value: u8,
    ) {
        match self
            .doc
            .resolve_address_with_channel_name(fixture_id, channel)
        {
            Ok(resolved_address) => {
                let universe = self
                    .universe_states
                    .get_mut(&resolved_address.universe)
                    .expect(
                        format!("universe states not found: {:?}", resolved_address.universe)
                            .as_str(),
                    ); // TODO: EngineMessageで通知する
                universe.set_value(resolved_address, value);
            }
            Err(e) => {
                self.message_tx
                    .send(EngineMessage::ErrorOccured(EngineError::ResolvingAddress {
                        fixture_id,
                        channel: channel.to_string(),
                        source: Box::new(e),
                    }))
                    .expect("failed to send message from engine");
            }
        }
    }
}

/// Message from the main thread to [`Engine`]
#[derive(Debug)]
pub enum EngineCommand {
    // Commands
    StartFunction(EffectId),
    /// 現在実行中のfunctionをstopする
    StopFunction,
    SetLiveValue {
        fixture_id: FixtureId,
        channel: String,
        value: u8,
    },
    AddPlugin(Box<dyn Plugin>),
    AddPluginDestination {
        plugin: OutputPluginId,
        dest_universe: UniverseId,
    },
    RemovePluginDestination {
        plugin: OutputPluginId,
        dest_universe: UniverseId,
    },
    Shutdown,

    // Events
    UniverseAdded(UniverseId),
    UniverseRemoved(UniverseId),
}

/// Message from [`Engine`] to the main thread
#[derive(Debug)]
pub enum EngineMessage {
    ErrorOccured(EngineError),
}

/// The errors occured in [`Engine`]
#[derive(Debug, Error)]
pub enum EngineError {
    #[error(
        "an error occured during resolving address of channel {channel} of fixture {fixture_id:?}: {source}"
    )]
    ResolvingAddress {
        fixture_id: FixtureId,
        channel: String,
        source: Box<dyn Error + Send + Sync>,
    },
    #[error(transparent)]
    ResolveAddress(#[from] ResolveError),
    #[error("")]
    RunningFunction {
        function_id: EffectId,
        source: Box<dyn Error + Send>,
    },
    #[error("")]
    SendingDmx {
        universe_id: UniverseId,
        plugin_id: OutputPluginId,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("function {function_id} not found in Doc")]
    FunctionNotFound { function_id: EffectId },
    #[error("no plugin {id:?} found")]
    OutputPluginNotFound { id: OutputPluginId },
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
        match resolved_address.channel_def.merge_mode() {
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
