use thiserror::Error;
use tokio::sync::watch;
use tokio::task;
use tracing::{debug, info, warn};

use crate::doc::{DocStateView, ResolveError, ResolvedAddress};
use crate::effects::{EffectCommand, EffectId, EffectRuntime};
use crate::fixture::{FixtureId, MergeMode};
use crate::plugins::{DmxFrame, OutputPluginId, Plugin, PluginMessage};
use crate::universe::UniverseId;
use std::collections::HashMap;
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
    //blocking_output_plugins: HashMap<OutputPluginId, Box<dyn BlockingPlugin>>,
    /// どのPluginがどのUniverseに出力するか
    //output_map: HashMap<OutputPluginId, HashSet<UniverseId>>,
    plugins: Vec<(UniverseId, watch::Sender<PluginMessage>)>,
    plugin_handles: Vec<tokio::task::JoinHandle<()>>,

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
            universe_states: HashMap::new(),
            plugins: Vec::new(),
            plugin_handles: Vec::new(),
            live_values: HashMap::new(),
            should_shutdown: false,
        }
    }

    /// Main entry point of [`Engine`].
    #[tokio::main]
    pub async fn start_loop(mut self) {
        info!("starting engine...");
        loop {
            self.handle_engine_commands();

            if self.should_shutdown {
                // TODO: JoinHandleとSenderが同じ順序というのは実装依存なので、一つのVecにまとめるなどしたい
                for (join, tx) in self
                    .plugin_handles
                    .into_iter()
                    .zip(self.plugins.iter().map(|(_, tx)| tx))
                {
                    tx.send(PluginMessage::Stop).unwrap();
                    join.await.expect("failed to join on plugin task");
                }
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
                EngineCommand::AddPlugin(plugin) => {
                    let univ = plugin.universe();
                    let (tx, rx) = watch::channel(PluginMessage::DmxFrame(DmxFrame::zeros()));
                    let handle =
                        task::spawn(async move { plugin.start(rx).await.expect("plugin exited") });
                    self.plugins.push((univ, tx));
                    self.plugin_handles.push(handle);
                }
                EngineCommand::Shutdown => self.should_shutdown = true,
            }
        }
    }

    fn run_active_function(&mut self) {
        let Some(commands) = self.active_runtime.as_mut().map(|rt| rt.run(TICK_DURATION)) else {
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

    /// Send dmx frame to all output plugins.
    fn dispatch_outputs(&mut self) {
        self.plugins.iter().for_each(|(univ, plugin)| {
            let frame = self
                .universe_states
                .get(univ)
                .expect("universe states not initialized"); // TODO: tracingに出力
            plugin
                .send(PluginMessage::DmxFrame(DmxFrame::from(frame.values)))
                .unwrap();
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
