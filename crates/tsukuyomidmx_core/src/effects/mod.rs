mod parallel;
mod sequence;
mod simple;

//pub use chaser::ChaserData;
//pub use collection::Collection;
//#[allow(unused)]
//pub(crate) use fader::Fader;
//pub use static_scene::SceneValue;
//pub use static_scene::StaticSceneData;

use derive_getters::Getters;
use serde::{Deserialize, Serialize};

use crate::doc::DocStateView;
use crate::effects::parallel::{ParallelEffectBody, ParallelEffectSpecBody};
use crate::effects::sequence::{SequenceEffectBody, SequenceEffectSpecBody, SequenceStep};
use crate::effects::simple::{SimpleEffectBody, SimpleEffectSpecBody};
use crate::fixture::FixtureId;
use std::collections::HashMap;
use std::time::Duration;

declare_id_newtype!(EffectSpecId);
declare_id_newtype!(EffectId);

pub(crate) trait EffectRuntime: Send {
    /// bodyのバリアントが自身と異なった場合はpanicして良い。
    fn run(
        &mut self,
        body: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand>;
}

pub(crate) trait StandAloneEffectRuntime: EffectRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand>;
}

/// bind_to()でFixtureに関連付けたあとのfunction.
///
/// Goboなどmodel-specificなチャンネルを制御する。
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct Effect {
    #[getter(copy)]
    id: EffectId,
    name: String,
    body: EffectBody,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectBody {
    Simple(SimpleEffectBody),
    Sequence(SequenceEffectBody),
    Parallel(ParallelEffectBody),
}

impl Effect {
    pub fn new_simple(
        name: impl Into<String>,
        values: impl Into<HashMap<(FixtureId, usize), u8>>,
    ) -> Effect {
        Effect {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Simple(SimpleEffectBody::new(values)),
        }
    }

    pub fn new_sequence(
        name: impl Into<String>,
        steps: impl Into<Vec<SequenceStep<EffectBody, EffectId>>>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Sequence(SequenceEffectBody::new(steps)),
        }
    }

    pub fn new_parallel(
        name: impl Into<String>,
        items: impl Into<Vec<EffectBodyOrReference<EffectBody, EffectId>>>,
    ) -> Self {
        Self {
            id: EffectId::new(),
            name: name.into(),
            body: EffectBody::Parallel(ParallelEffectBody::new(items)),
        }
    }

    pub(crate) fn create_standalone_runtime(
        &self,
        doc: DocStateView,
    ) -> Box<dyn StandAloneEffectRuntime> {
        self.body.create_standalone_runtime(self.id, doc)
    }
}

impl EffectBody {
    /// infallible
    fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        match &self {
            EffectBody::Simple(fun) => fun.create_runtime(),
            EffectBody::Sequence(fun) => fun.create_runtime(doc),
            EffectBody::Parallel(fun) => fun.create_runtime(doc),
        }
    }

    fn create_standalone_runtime(
        &self,
        self_id: EffectId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneEffectRuntime> {
        match &self {
            EffectBody::Simple(fun) => fun.create_runtime_standalone(self_id),
            EffectBody::Sequence(fun) => fun.create_runtime_standalone(self_id, doc),
            EffectBody::Parallel(fun) => fun.create_runtime_standalone(self_id, doc),
        }
    }
}

/// bind_to()でFixtureに関連付けられる前のfunction.
///
/// Dimmer, Colorなどmodel-agnosticなチャンネルを制御する。
#[derive(Debug, Serialize, Deserialize)]
pub struct EffectSpec {
    id: EffectSpecId,
    name: String,
    body: EffectSpecBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EffectSpecBody {
    Simple(SimpleEffectSpecBody),
    Sequence(SequenceEffectSpecBody),
    Parallel(ParallelEffectSpecBody),
}

impl EffectSpec {
    pub fn id(&self) -> EffectSpecId {
        self.id
    }

    pub fn bind_to(
        &self,
        name: impl Into<String>,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<Effect> {
        match &self.body {
            EffectSpecBody::Simple(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Effect {
                        id: EffectId::new(),
                        name: name.into(),
                        body: EffectBody::Simple(fun),
                    })
                } else {
                    None
                }
            }
            EffectSpecBody::Sequence(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Effect {
                        id: EffectId::new(),
                        name: name.into(),
                        body: EffectBody::Sequence(fun),
                    })
                } else {
                    None
                }
            }
            EffectSpecBody::Parallel(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Effect {
                        id: EffectId::new(),
                        name: name.into(),
                        body: EffectBody::Parallel(fun),
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub struct Diagnostics {
    inner: Vec<DiagnosticItem>,
}

struct DiagnosticItem {
    message: String,
}

impl Diagnostics {
    pub fn push_err(&mut self, message: impl Into<String>) {
        self.inner.push(DiagnosticItem {
            message: message.into(),
        });
    }
}

/// [`FunctionRuntime::run()`] returns this and [`Engine`][crate::engine::Engine] evaluates the command
#[derive(Debug)]
pub enum EffectCommand {
    /// if the function is already started, `Engine` do nothing.
    StartEffect(EffectId),
    /// 実行中のFunctionをstopする
    StopEffect,
    WriteUniverse {
        fixture_id: FixtureId,
        channel: usize,
        value: u8,
    },
    StartFade {
        from_id: EffectId,
        to_id: EffectId,
        chaser_id: EffectId,
        duration: Duration,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectBodyOrReference<T, U> {
    Body(T),
    Reference(U),
}

impl EffectBodyOrReference<EffectBody, EffectId> {
    fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        match self {
            Self::Body(body) => body.create_runtime(doc),
            Self::Reference(id) => {
                doc.with_functions(|it| it.get(id).unwrap().body.create_runtime(doc.clone()))
            }
        }
    }
}
