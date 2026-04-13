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
use crate::fixture::FixtureId;
use crate::functions::parallel::{ParallelFunctionBody, ParallelFunctionPrototypeBody};
use crate::functions::sequence::{
    SequenceFunctionBody, SequenceFunctionPrototypeBody, SequenceStep,
};
use crate::functions::simple::{SimpleFunctionBody, SimpleFunctionPrototypeBody};
use std::collections::HashMap;
use std::time::Duration;

declare_id_newtype!(FunctionPrototypeId);
declare_id_newtype!(AppliedFunctionId);

pub(crate) trait FunctionRuntime: Send {
    /// bodyのバリアントが自身と異なった場合はpanicして良い。
    fn run(
        &mut self,
        body: &FunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand>;
}

pub(crate) trait StandAloneFunctionRuntime: FunctionRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<FunctionCommand>;
}

/// bind_to()でFixtureに関連付けたあとのfunction.
///
/// Goboなどmodel-specificなチャンネルを制御する。
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct Function {
    #[getter(copy)]
    id: AppliedFunctionId,
    name: String,
    body: FunctionBody,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionBody {
    Simple(SimpleFunctionBody),
    Sequence(SequenceFunctionBody),
    Parallel(ParallelFunctionBody),
}

impl Function {
    pub fn new_simple(
        name: impl Into<String>,
        values: impl Into<HashMap<(FixtureId, usize), u8>>,
    ) -> Function {
        Function {
            id: AppliedFunctionId::new(),
            name: name.into(),
            body: FunctionBody::Simple(SimpleFunctionBody::new(values)),
        }
    }

    pub fn new_sequence(
        name: impl Into<String>,
        steps: impl Into<Vec<SequenceStep<FunctionBody, AppliedFunctionId>>>,
    ) -> Self {
        Self {
            id: AppliedFunctionId::new(),
            name: name.into(),
            body: FunctionBody::Sequence(SequenceFunctionBody::new(steps)),
        }
    }

    pub(crate) fn create_standalone_runtime(
        &self,
        doc: DocStateView,
    ) -> Box<dyn StandAloneFunctionRuntime> {
        self.body.create_standalone_runtime(self.id, doc)
    }
}

impl FunctionBody {
    /// infallible
    fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        match &self {
            FunctionBody::Simple(fun) => fun.create_runtime(),
            FunctionBody::Sequence(fun) => fun.create_runtime(doc),
            FunctionBody::Parallel(fun) => fun.create_runtime(),
        }
    }

    fn create_standalone_runtime(
        &self,
        self_id: AppliedFunctionId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneFunctionRuntime> {
        match &self {
            FunctionBody::Simple(fun) => fun.create_runtime_standalone(self_id),
            FunctionBody::Sequence(fun) => fun.create_runtime_standalone(self_id, doc),
            FunctionBody::Parallel(fun) => fun.create_runtime_standalone(self_id),
        }
    }
}

/// bind_to()でFixtureに関連付けられる前のfunction.
///
/// Dimmer, Colorなどmodel-agnosticなチャンネルを制御する。
#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionPrototype {
    id: FunctionPrototypeId,
    name: String,
    body: FunctionPrototypeBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FunctionPrototypeBody {
    Simple(SimpleFunctionPrototypeBody),
    Sequence(SequenceFunctionPrototypeBody),
    Parallel(ParallelFunctionPrototypeBody),
}

impl FunctionPrototype {
    pub fn id(&self) -> FunctionPrototypeId {
        self.id
    }

    pub fn bind_to(
        &self,
        name: impl Into<String>,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<Function> {
        match &self.body {
            FunctionPrototypeBody::Simple(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function {
                        id: AppliedFunctionId::new(),
                        name: name.into(),
                        body: FunctionBody::Simple(fun),
                    })
                } else {
                    None
                }
            }
            FunctionPrototypeBody::Sequence(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function {
                        id: AppliedFunctionId::new(),
                        name: name.into(),
                        body: FunctionBody::Sequence(fun),
                    })
                } else {
                    None
                }
            }
            FunctionPrototypeBody::Parallel(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function {
                        id: AppliedFunctionId::new(),
                        name: name.into(),
                        body: FunctionBody::Parallel(fun),
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
pub enum FunctionCommand {
    /// if the function is already started, `Engine` do nothing.
    StartFunction(AppliedFunctionId),
    /// 実行中のFunctionをstopする
    StopFuntion,
    WriteUniverse {
        fixture_id: FixtureId,
        channel: usize,
        value: u8,
    },
    StartFade {
        from_id: AppliedFunctionId,
        to_id: AppliedFunctionId,
        chaser_id: AppliedFunctionId,
        duration: Duration,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum FunctionBodyOrId<T, U> {
    Body(T),
    Id(U),
}

impl FunctionBodyOrId<FunctionBody, AppliedFunctionId> {
    fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        match self {
            Self::Body(body) => body.create_runtime(doc),
            Self::Id(id) => {
                doc.with_functions(|it| it.get(id).unwrap().body.create_runtime(doc.clone()))
            }
        }
    }
}
