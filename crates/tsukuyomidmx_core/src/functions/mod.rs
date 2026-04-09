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
    fn run(&mut self, doc: DocStateView, elapsed: Duration) -> Vec<FunctionCommand>;
}

/// bind_to()でFixtureに関連付けたあとのfunction.
///
/// Goboなどmodel-specificなチャンネルを制御する。
#[derive(Debug)]
pub enum Function {
    Simple(SimpleFunction),
    Sequence(SequenceFunction),
    Parallel(ParallelFunction),
}

impl Function {
    /// infallible
    pub fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        match self {
            Function::Simple(fun) => fun.create_runtime_inner(),
            Function::Sequence(fun) => fun.create_runtime_inner(doc),
            Function::Parallel(fun) => fun.create_runtime_inner(),
        }
    }

    pub fn id(&self) -> AppliedFunctionId {
        match self {
            Function::Simple(fun) => fun.id,
            Function::Sequence(fun) => fun.id,
            Function::Parallel(fun) => fun.id,
        }
    }
}

/// bind_to()でFixtureに関連付けられる前のfunction.
///
/// Dimmer, Colorなどmodel-agnosticなチャンネルを制御する。
#[derive(Debug)]
pub enum FunctionPrototype {
    Simple(SimpleFunctionPrototype),
    Sequence(SequenceFunctionPrototype),
    Parallel(ParallelFunctionPrototype),
}

impl FunctionPrototype {
    pub fn bind_to(
        self,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
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

#[derive(Debug)]
pub struct SimpleFunctionPrototype {
    id: FunctionPrototypeId,
    dimmer: Option<u8>,
    color: Option<[u8; 3]>,
}

#[derive(Debug)]
pub struct SimpleFunction {
    id: AppliedFunctionId,
    /// (id, offset) -> value
    values: HashMap<(FixtureId, usize), u8>,
}

pub struct SimpleFunctionRuntime(AppliedFunctionId);

impl SimpleFunctionPrototype {
    fn bind_to_inner(
        &self,
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
    pub fn new(values: HashMap<(FixtureId, usize), u8>) -> Function {
        Function::Simple(Self {
            id: AppliedFunctionId::new(),
            values,
        })
    }

    fn create_runtime_inner(&self) -> Box<dyn FunctionRuntime> {
        Box::new(SimpleFunctionRuntime(self.id))
    }
}

impl FunctionRuntime for SimpleFunctionRuntime {
    fn fun_id(&self) -> AppliedFunctionId {
        self.0
    }

    fn run(&mut self, doc: DocStateView, _elapsed: Duration) -> Vec<FunctionCommand> {
        doc.with_functions(|it| {
            let Function::Simple(fun) = it.get(&self.0).unwrap() else {
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
        })
    }
}

#[derive(Debug)]
pub struct SequenceFunctionPrototype {
    // TODO: Box<dyn AppliedFunction>のみ or Box<dyn FunctionPrototype>のみにしたい
    steps: Vec<SequenceStep<FunctionPrototype, FunctionPrototypeId>>,
}

#[derive(Debug)]
pub struct SequenceFunction {
    id: AppliedFunctionId,
    steps: Vec<SequenceStep<Function, AppliedFunctionId>>,
}

#[derive(Debug)]
pub struct SequenceStep<T, U> {
    /// fade_in, fade_outを除いた時間
    duration: Duration,
    fade_in: Duration,
    fade_out: Duration,
    body: FunctionBodyOrId<T, U>, // TODO: 他のFunctionのIdを持っておいたほうが良いのでは？
}

#[derive(Debug)]
enum FunctionBodyOrId<T, U> {
    Body(T),
    Id(U),
}

impl FunctionBodyOrId<Function, AppliedFunctionId> {
    /// Bodyのときはcreate_runtime()を呼ぶだけ、Idだったらdocを使う
    fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        match self {
            Self::Body(fun) => fun.create_runtime(doc),
            Self::Id(fun_id) => {
                doc.with_functions(|it| it.get(fun_id).unwrap().create_runtime(doc.clone()))
            }
        }
    }
}

pub struct SequenceFunctionRuntime {
    fun_id: AppliedFunctionId,
    time_to_next_step: Duration,
    current_step: usize,
    current_step_runtime: Box<dyn FunctionRuntime>,
}

impl SequenceFunctionPrototype {
    fn bind_to_inner(
        &self,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<SequenceFunction> {
        let steps = self.steps.iter().map(|step| match &step.body {
            FunctionBodyOrId::Body(fun) => match fun {
                FunctionPrototype::Simple(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(Function::Simple(fun)),
                    }),
                FunctionPrototype::Sequence(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(Function::Sequence(fun)),
                    }),
                FunctionPrototype::Parallel(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(Function::Parallel(fun)),
                    }),
            },
            FunctionBodyOrId::Id(_fun_id) => todo!(),
        });

        let steps = steps.collect::<Option<Vec<_>>>()?;
        Some(SequenceFunction {
            id: AppliedFunctionId::new(),
            steps,
        })
    }
}

impl SequenceFunction {
    fn create_runtime_inner(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        Box::new(SequenceFunctionRuntime {
            fun_id: self.id,
            time_to_next_step: self.steps.get(0).unwrap().total_duration(),
            current_step: 0,
            current_step_runtime: self.steps.get(0).unwrap().body.create_runtime(doc),
        })
    }
}

impl FunctionRuntime for SequenceFunctionRuntime {
    fn fun_id(&self) -> AppliedFunctionId {
        self.fun_id
    }

    fn run(&mut self, doc: DocStateView, elapsed: Duration) -> Vec<FunctionCommand> {
        // TODO: fade_inとfade_out
        let mut commands = self.current_step_runtime.run(doc.clone(), elapsed);

        if self.time_to_next_step >= elapsed {
            // ステップ継続
            self.time_to_next_step -= elapsed;
            return commands;
        }
        doc.with_functions(|it| {
            let Function::Sequence(fun) = it.get(&self.fun_id).unwrap() else {
                unreachable!()
            };

            if fun.steps.len() == self.current_step {
                //全ステップ終わった
                commands.push(FunctionCommand::StopFuntion);
            } else {
                // 次のステップ

                let next_step = fun.steps.get(self.current_step).unwrap();
                self.current_step += 1;
                self.time_to_next_step =
                    next_step.total_duration() + elapsed - self.time_to_next_step;
                self.current_step_runtime = next_step.body.create_runtime(doc.clone());
            }
        });

        commands
    }
}

impl<T, U> SequenceStep<T, U> {
    /// fade_inとfade_out含めた時間
    fn total_duration(&self) -> Duration {
        self.fade_in + self.duration + self.fade_out
    }
}

#[derive(Debug)]
pub struct ParallelFunctionPrototype {}

#[derive(Debug)]
pub struct ParallelFunction {
    id: AppliedFunctionId,
}

pub struct ParallelFunctionRuntime {
    fun_id: AppliedFunctionId,
}

impl ParallelFunctionPrototype {
    fn bind_to_inner(
        &self,
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

    fn run(&mut self, _doc: DocStateView, _elapsed: Duration) -> Vec<FunctionCommand> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_run_works() {}
}
