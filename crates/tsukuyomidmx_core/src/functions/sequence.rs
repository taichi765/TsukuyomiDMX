use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceFunctionPrototypeBody {
    // TODO: Box<dyn AppliedFunction>のみ or Box<dyn FunctionPrototype>のみにしたい
    steps: Vec<SequenceStep<FunctionPrototypeBody, FunctionPrototypeId>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceFunctionBody {
    steps: Vec<SequenceStep<FunctionBody, AppliedFunctionId>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceStep<T, U> {
    /// fade_in, fade_outを除いた時間
    duration: Duration,
    fade_in: Duration,
    fade_out: Duration,
    body: FunctionBodyOrId<T, U>,
}

/// スタンドアロンのランタイム
pub struct StandAloneSequenceFunctionRuntime {
    fun_id: AppliedFunctionId,
    inner: SequenceFunctionRuntime,
}

pub struct SequenceFunctionRuntime {
    time_to_next_step: Duration,
    current_step_num: usize,
    current_step_runtime: Box<dyn FunctionRuntime>,
}

impl SequenceFunctionPrototypeBody {
    pub(super) fn bind_to_inner(
        &self,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<SequenceFunctionBody> {
        let steps = self.steps.iter().map(|step| match &step.body {
            FunctionBodyOrId::Body(fun) => match fun {
                FunctionPrototypeBody::Simple(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(FunctionBody::Simple(fun)),
                    }),
                FunctionPrototypeBody::Sequence(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(FunctionBody::Sequence(fun)),
                    }),
                FunctionPrototypeBody::Parallel(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: FunctionBodyOrId::Body(FunctionBody::Parallel(fun)),
                    }),
            },
            FunctionBodyOrId::Id(_fun_id) => todo!(),
        });

        let steps = steps.collect::<Option<Vec<_>>>()?;
        Some(SequenceFunctionBody { steps })
    }
}

impl SequenceFunctionBody {
    pub(super) fn new(
        steps: impl Into<Vec<SequenceStep<FunctionBody, AppliedFunctionId>>>,
    ) -> Self {
        Self {
            steps: steps.into(),
        }
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: AppliedFunctionId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneFunctionRuntime> {
        Box::new(StandAloneSequenceFunctionRuntime {
            fun_id: self_id,
            inner: SequenceFunctionRuntime {
                time_to_next_step: self.steps.get(0).unwrap().total_duration(),
                current_step_num: 0,
                current_step_runtime: self.steps.get(0).unwrap().body.create_runtime(doc),
            },
        })
    }

    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        Box::new(SequenceFunctionRuntime {
            time_to_next_step: self.steps.get(0).unwrap().total_duration(),
            current_step_num: 0,
            current_step_runtime: self.steps.get(0).unwrap().body.create_runtime(doc),
        })
    }
}

impl FunctionRuntime for SequenceFunctionRuntime {
    fn run(
        &mut self,
        this: &FunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        let FunctionBody::Sequence(this) = this else {
            unreachable!()
        };

        // TODO: fade_inとfade_out
        let mut commands = self.run_current_step(this, elapsed, doc.clone());

        if self.time_to_next_step >= elapsed {
            // ステップ継続
            self.time_to_next_step -= elapsed;
            return commands;
        }

        if this.steps.len() == self.current_step_num {
            //全ステップ終わった
            commands.push(FunctionCommand::StopFuntion);
        } else {
            // 次のステップ

            let next_step = this.steps.get(self.current_step_num).unwrap();
            self.current_step_num += 1;
            self.time_to_next_step = next_step.total_duration() + elapsed - self.time_to_next_step;
            self.current_step_runtime = next_step.body.create_runtime(doc);
        }

        commands
    }
}

impl FunctionRuntime for StandAloneSequenceFunctionRuntime {
    fn run(
        &mut self,
        body: &FunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        self.inner.run(body, elapsed, doc)
    }
}

impl StandAloneFunctionRuntime for StandAloneSequenceFunctionRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<FunctionCommand> {
        doc.with_functions(|it| {
            let this = &it.get(&self.fun_id).unwrap().body;
            self.inner.run(this, elapsed, doc.clone())
        })
    }
}

impl SequenceFunctionRuntime {
    fn run_current_step(
        &mut self,
        this: &SequenceFunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        match &this.steps.get(self.current_step_num).unwrap().body {
            FunctionBodyOrId::Body(body) => self.current_step_runtime.run(body, elapsed, doc),
            FunctionBodyOrId::Id(id) => doc.with_functions(|it| {
                let body = &it.get(id).unwrap().body;
                self.current_step_runtime.run(body, elapsed, doc.clone())
            }),
        }
    }
}

impl<T, U> SequenceStep<T, U> {
    /// fade_inとfade_out含めた時間
    fn total_duration(&self) -> Duration {
        self.fade_in + self.duration + self.fade_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_function_is_serialized_and_deserialized_correctly() {
        let fxt_id = FixtureId::new();
        let simple = Function::new_simple(
            "Scene 2",
            vec![((fxt_id, 3usize), 123u8), ((fxt_id, 4), 100)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );
        let fun = Function::new_sequence(
            "Scene 1",
            vec![
                SequenceStep {
                    duration: Duration::from_millis(500),
                    fade_in: Duration::ZERO,
                    fade_out: Duration::ZERO,
                    body: FunctionBodyOrId::Body(FunctionBody::Simple(SimpleFunctionBody::new(
                        vec![((fxt_id, 0usize), 255u8), ((fxt_id, 1), 200)]
                            .into_iter()
                            .collect::<HashMap<_, _>>(),
                    ))),
                },
                SequenceStep {
                    duration: Duration::from_millis(700),
                    fade_in: Duration::from_millis(100),
                    fade_out: Duration::ZERO,
                    body: FunctionBodyOrId::Id(simple.id()),
                },
            ],
        );

        let json = serde_json::to_string_pretty(&fun).unwrap();
        println!("{}", json);

        let deserialized: Function = serde_json::from_str(&json).unwrap();

        assert_eq!(fun, deserialized);
    }
}
