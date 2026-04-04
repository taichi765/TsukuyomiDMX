//mod chaser;
//mod collection;
//mod fader;
//mod static_scene;
//mod timeline;

//pub use chaser::ChaserData;
//pub use collection::Collection;
//#[allow(unused)]
//pub(crate) use fader::Fader;
//pub use static_scene::SceneValue;
//pub use static_scene::StaticSceneData;

use crate::doc::DocStateView;
use crate::fixture::FixtureId;
use std::collections::HashMap;
use std::time::Duration;

declare_id_newtype!(FunctionPrototypeId);
declare_id_newtype!(AppliedFunctionId);

pub trait FunctionRuntime: Send {
    fn fun_id(&self) -> AppliedFunctionId;
    fn run(&mut self, function: &Function, elapsed: Duration) -> Vec<FunctionCommand>;
}

/// bind_to()でFixtureに関連付けたあとのfunction.
///
/// Goboなどmodel-specificなチャンネルを制御する。
pub enum Function {
    Simple(SimpleFunction),
    Sequence(SequenceFunction),
    Parallel(ParallelFunction),
}

impl Function {
    /// infallible
    pub fn create_runtime(&self) -> Box<dyn FunctionRuntime> {
        match self {
            Function::Simple(fun) => fun.create_runtime_inner(),
            Function::Sequence(fun) => fun.create_runtime_inner(),
            Function::Parallel(fun) => fun.create_runtime_inner(),
        }
    }
}

/// bind_to()でFixtureに関連付けられる前のfunction.
///
/// Dimmer, Colorなどmodel-agnosticなチャンネルを制御する。
pub enum FunctionPrototype {
    Simple(SimpleFunctionPrototype),
    Sequence(SequenceFunctionPrototype),
    Parallel(ParallelFunctionPrototype),
}

impl FunctionPrototype {
    fn bind_to(
        self,
        args: impl Iterator<Item = Vec<FixtureId>>,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<Function> {
        match self {
            FunctionPrototype::Simple(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function::Simple(fun))
                } else {
                    None
                }
            }
            FunctionPrototype::Sequence(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function::Sequence(fun))
                } else {
                    None
                }
            }
            FunctionPrototype::Parallel(p) => {
                if let Some(fun) = p.bind_to_inner(args, doc, diag) {
                    Some(Function::Parallel(fun))
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
pub enum FunctionCommand {
    /// if the function is already started, `Engine` do nothing.
    StartFunction(AppliedFunctionId),
    /// if the function is already stoped, `Engine` do nothing.
    StopFuntion(AppliedFunctionId),
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

pub struct SimpleFunctionPrototype {
    id: FunctionPrototypeId,
    dimmer: Option<u8>,
    color: Option<[u8; 3]>,
}

pub struct SimpleFunction {
    id: AppliedFunctionId,
    /// (id, offset) -> value
    values: HashMap<(FixtureId, usize), u8>,
}

pub struct SimpleFunctionRuntime(AppliedFunctionId);

impl SimpleFunctionPrototype {
    fn bind_to_inner(
        self,
        mut args: impl Iterator<Item = Vec<FixtureId>>,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<SimpleFunction> {
        let mut values = HashMap::new();
        let mut has_error = false;

        let Some(fixtures) = args.next() else {
            diag.push_err("no argument provided");
            return None;
        };
        if args.next().is_some() {
            diag.push_err("too many arguments provided");
            return None;
        }

        for fxt_id in fixtures {
            doc.with_fixtures_and_defs(|fxts, defs| {
                let fxt = fxts.get(&fxt_id).unwrap();
                let def = defs.get(fxt.fixture_def()).unwrap();

                if let Some(dimmer) = self.dimmer {
                    if let Some(offset) = def.find_dimmer_channel_in_mode(fxt.fixture_mode()) {
                        values.insert((fxt_id, offset), dimmer);
                    } else {
                        diag.push_err(format!("fixture {fxt_id:?} doesn't have dimmer channel"));
                        has_error = true;
                    };
                }

                if let Some(color) = self.color {
                    if let Some(rgb_offset) = def.find_rgb_channel_in_mode(fxt.fixture_mode()) {
                        rgb_offset
                            .into_iter()
                            .zip(color)
                            .for_each(|(offset, color)| {
                                values.insert((fxt_id, offset), color);
                            });
                    } else {
                        diag.push_err("this fixture doesn't have rgb channel");
                        has_error = true;
                    }
                }
            });
        }

        if has_error {
            return None;
        }

        Some(SimpleFunction {
            id: AppliedFunctionId::new(),
            values,
        })
    }
}

impl SimpleFunction {
    fn create_runtime_inner(&self) -> Box<dyn FunctionRuntime> {
        Box::new(SimpleFunctionRuntime(self.id))
    }
}

impl FunctionRuntime for SimpleFunctionRuntime {
    fn fun_id(&self) -> AppliedFunctionId {
        self.0
    }

    fn run(&mut self, function: &Function, _elapsed: Duration) -> Vec<FunctionCommand> {
        let Function::Simple(fun) = function else {
            unreachable!()
        };
        fun.values
            .iter()
            .fold(Vec::new(), |mut acc, ((fxt_id, offset), val)| {
                acc.push(FunctionCommand::WriteUniverse {
                    fixture_id: *fxt_id,
                    channel: *offset,
                    value: *val,
                });
                acc
            })
    }
}

pub struct SequenceFunctionPrototype {
    // TODO: Box<dyn AppliedFunction>のみ or Box<dyn FunctionPrototype>のみにしたい
    steps: Vec<SequenceStep<FunctionPrototype>>,
}

pub struct SequenceFunction {
    steps: Vec<SequenceStep<Function>>,
}

pub struct SequenceFunctionRuntime {
    fun_id: AppliedFunctionId,
}

pub struct SequenceStep<T> {
    duration: Duration,
    fade_in: Duration,
    fade_out: Duration,
    body: T,
}

impl SequenceFunctionPrototype {
    fn bind_to_inner(
        self,
        _args: impl Iterator<Item = Vec<FixtureId>>,
        _doc: DocStateView,
        _diag: &mut Diagnostics,
    ) -> Option<SequenceFunction> {
        todo!()
    }
}

impl SequenceFunction {
    fn create_runtime_inner(&self) -> Box<dyn FunctionRuntime> {
        todo!()
    }
}

impl FunctionRuntime for SequenceFunctionRuntime {
    fn fun_id(&self) -> AppliedFunctionId {
        self.fun_id
    }

    fn run(&mut self, _function: &Function, _elapsed: Duration) -> Vec<FunctionCommand> {
        todo!()
    }
}

pub struct ParallelFunctionPrototype {}

pub struct ParallelFunction {}

pub struct ParallelFunctionRuntime {
    fun_id: AppliedFunctionId,
}

impl ParallelFunctionPrototype {
    fn bind_to_inner(
        self,
        _args: impl Iterator<Item = Vec<FixtureId>>,
        _doc: DocStateView,
        _diag: &mut Diagnostics,
    ) -> Option<ParallelFunction> {
        todo!()
    }
}

impl ParallelFunction {
    fn create_runtime_inner(&self) -> Box<dyn FunctionRuntime> {
        todo!()
    }
}

impl FunctionRuntime for ParallelFunctionRuntime {
    fn fun_id(&self) -> AppliedFunctionId {
        self.fun_id
    }

    fn run(&mut self, _function: &Function, _elapsed: Duration) -> Vec<FunctionCommand> {
        todo!()
    }
}
