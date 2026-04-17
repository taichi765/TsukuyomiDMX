use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceEffectSpecBody {
    steps: Vec<SequenceStep<EffectSpecBody, EffectSpecId>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceEffectBody {
    steps: Vec<SequenceStep<EffectBody, EffectId>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceStep<T, U> {
    /// fade_in, fade_outを除いた時間
    duration: Duration,
    fade_in: Duration,
    fade_out: Duration,
    body: EffectBodyOrReference<T, U>,
}

/// スタンドアロンのランタイム
pub struct StandAloneSequenceEffectRuntime {
    fun_id: EffectId,
    inner: SequenceEffectRuntime,
}

pub struct SequenceEffectRuntime {
    time_to_next_step: Duration,
    current_step_num: usize,
    current_step_runtime: Box<dyn EffectRuntime>,
}

impl SequenceEffectSpecBody {
    pub(super) fn bind_to_inner(
        &self,
        args: impl Iterator<Item = Vec<FixtureId>> + Clone,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<SequenceEffectBody> {
        let steps = self.steps.iter().map(|step| match &step.body {
            EffectBodyOrReference::Body(fun) => match fun {
                EffectSpecBody::Simple(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: EffectBodyOrReference::Body(EffectBody::Simple(fun)),
                    }),
                EffectSpecBody::Sequence(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: EffectBodyOrReference::Body(EffectBody::Sequence(fun)),
                    }),
                EffectSpecBody::Parallel(fun) => fun
                    .bind_to_inner(args.clone(), doc.clone(), diag)
                    .map(|fun| SequenceStep {
                        duration: step.duration,
                        fade_in: step.fade_in,
                        fade_out: step.fade_out,
                        body: EffectBodyOrReference::Body(EffectBody::Parallel(fun)),
                    }),
            },
            EffectBodyOrReference::Reference(_fun_id) => todo!(),
        });

        let steps = steps.collect::<Option<Vec<_>>>()?;
        Some(SequenceEffectBody { steps })
    }
}

impl SequenceEffectBody {
    pub(super) fn new(steps: impl Into<Vec<SequenceStep<EffectBody, EffectId>>>) -> Self {
        Self {
            steps: steps.into(),
        }
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: EffectId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneEffectRuntime> {
        Box::new(StandAloneSequenceEffectRuntime {
            fun_id: self_id,
            inner: SequenceEffectRuntime {
                time_to_next_step: self.steps.get(0).unwrap().total_duration(),
                current_step_num: 0,
                current_step_runtime: self.steps.get(0).unwrap().body.create_runtime(doc),
            },
        })
    }

    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        Box::new(SequenceEffectRuntime {
            time_to_next_step: self.steps.get(0).unwrap().total_duration(),
            current_step_num: 0,
            current_step_runtime: self.steps.get(0).unwrap().body.create_runtime(doc),
        })
    }
}

impl EffectRuntime for SequenceEffectRuntime {
    fn run(
        &mut self,
        this: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        let EffectBody::Sequence(this) = this else {
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
            commands.push(EffectCommand::StopEffect);
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

impl EffectRuntime for StandAloneSequenceEffectRuntime {
    fn run(
        &mut self,
        body: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        self.inner.run(body, elapsed, doc)
    }
}

impl StandAloneEffectRuntime for StandAloneSequenceEffectRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand> {
        doc.with_functions(|it| {
            let this = &it.get(&self.fun_id).unwrap().body;
            self.inner.run(this, elapsed, doc.clone())
        })
    }
}

impl SequenceEffectRuntime {
    fn run_current_step(
        &mut self,
        this: &SequenceEffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        match &this.steps.get(self.current_step_num).unwrap().body {
            EffectBodyOrReference::Body(body) => self.current_step_runtime.run(body, elapsed, doc),
            EffectBodyOrReference::Reference(id) => doc.with_functions(|it| {
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
        let simple = Effect::new_simple(
            "Scene 2",
            vec![((fxt_id, 3usize), 123u8), ((fxt_id, 4), 100)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );
        let fun = Effect::new_sequence(
            "Scene 1",
            vec![
                SequenceStep {
                    duration: Duration::from_millis(500),
                    fade_in: Duration::ZERO,
                    fade_out: Duration::ZERO,
                    body: EffectBodyOrReference::Body(EffectBody::Simple(SimpleEffectBody::new(
                        vec![((fxt_id, 0usize), 255u8), ((fxt_id, 1), 200)]
                            .into_iter()
                            .collect::<HashMap<_, _>>(),
                    ))),
                },
                SequenceStep {
                    duration: Duration::from_millis(700),
                    fade_in: Duration::from_millis(100),
                    fade_out: Duration::ZERO,
                    body: EffectBodyOrReference::Reference(simple.id()),
                },
            ],
        );

        let json = serde_json::to_string_pretty(&fun).unwrap();
        println!("{}", json);

        let deserialized: Effect = serde_json::from_str(&json).unwrap();

        assert_eq!(fun, deserialized);
    }
}
