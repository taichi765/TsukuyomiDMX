use super::*;

impl EffectRegistry<SequenceEffectSpecBody, SequenceEffectTemplateBody> for DocStateView {
    fn with_spec<F, R>(&self, spec_id: EffectSpecId, f: F) -> R
    where
        F: FnOnce(&SequenceEffectSpecBody) -> R,
    {
        self.with_effect_specs(|it| {
            let EffectSpecBody::Sequence(body) = &it.get(&spec_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }

    fn with_template<F, R>(&self, tmpl_id: EffectTemplateId, f: F) -> R
    where
        F: FnOnce(&SequenceEffectTemplateBody) -> R,
    {
        self.with_effect_templates(|it| {
            let EffectTemplateBody::Sequence(body) = &it.get(&tmpl_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceTemplateStepBase<Body, Id> {
    /// `fade_in`を除いた時間
    hold: Expression,
    fade_in: Expression,
    body: EffectBodyOrReference<Body, Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceEffectSpecBody {
    props: HashMap<String, Type>,
    steps: Vec<SequenceSpecStep>,
}

type SequenceSpecStep = SequenceTemplateStepBase<EffectSpecBody, EffectSpecId>;

impl SequenceEffectSpecBody {
    fn resolve_props(&self, given_props: HashMap<String, Value>) -> Vec<ResolvedSequenceStep> {
        resolve_steps(&self.steps, given_props, |body, props| {
            body.resolve_props(props)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceEffectTemplateBody {
    FromSpec {
        spec_id: EffectSpecId,
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
    },
    // TODO: FromTemplate(),
    New {
        props: HashMap<String, Type>,
        steps: Vec<SequenceTemplateStep>,
        fixtures: FixtureQuery, // TODO: これいる？
    },
}

type SequenceTemplateStep = SequenceTemplateStepBase<EffectTemplateBody, EffectTemplateId>;

impl SequenceEffectTemplateBody {
    pub fn from_spec(
        spec_id: EffectSpecId,
        rg: impl EffectRegistry<SequenceEffectSpecBody, SequenceEffectTemplateBody>,
    ) -> Result<Self, FromSpecError> {
        // FIXME: ここでget(spec_id)をunwrap()しちゃだめな気がしなくもない
        let spec_props = rg.with_spec(spec_id, |spec| {
            let ret = spec
                .props
                .iter()
                .map(|(name, typ)| (name.to_owned(), Expression::Value(typ.default_value())))
                .collect();
            Ok::<_, _>(ret)
        })?;

        Ok(Self::FromSpec {
            spec_id,
            spec_props,
            props: HashMap::new(),
        })
    }

    fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        rg: impl EffectRegistry<SequenceEffectSpecBody, SequenceEffectTemplateBody>,
    ) -> Vec<ResolvedSequenceStep> {
        match self {
            Self::FromSpec {
                spec_id,
                spec_props,
                props,
            } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be apllied");

                let resolved_spec_props: HashMap<_, _> = spec_props
                    .iter()
                    .map(|(p_name, p_val)| match p_val {
                        Expression::Prop(p) => {
                            (p_name.to_owned(), given_props.get(p).cloned().unwrap())
                        }
                        Expression::Value(val) => (p_name.to_owned(), val.clone()),
                    })
                    .collect();

                rg.with_spec(*spec_id, |spec| spec.resolve_props(resolved_spec_props))
            }
            Self::New {
                props,
                steps,
                fixtures,
            } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be applied");

                resolve_steps(steps, given_props, |body, props| body.resolve_props(props))
            }
        }
    }
}

pub enum FromSpecError {
    SpecNotFound(EffectSpecId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceEffectBody {
    //TODO: FromSpec(SequenceEffectTemplateBody, HashMap<String, Value>),
    FromTemplate(EffectTemplateId, HashMap<String, Value>),
    New(Vec<SequenceStep>),
}

impl SequenceEffectBody {
    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate(tmpl_id, tmpl_props) => doc.with_effect_templates(|it| {
                let EffectTemplateBody::Sequence(tmpl) = &it.get(tmpl_id).unwrap().body else {
                    unreachable!()
                };

                let steps = tmpl.resolve_props(tmpl_props.clone(), doc.clone());
                Box::new(SequenceEffectRuntime::new(steps))
            }),
            Self::New(steps) => {
                // TODO: SequenceEffectTemplateBody::resolve_props()と重複しているコードがある
                // TODO: SimpleEffectRuntime::empty()みたいなやつ作る。作るときにfixturesを渡す。
                let steps = steps
                    .iter()
                    .scan(Vec::new(), |prev_last_frame, cur_step| {
                        let rt = cur_step.body.create_runtime(doc.clone());
                        let cur_first_frame = rt.first_frame_hint();
                        *prev_last_frame = rt.last_frame_hint();
                        Some(ResolvedSequenceStep {
                            fade_in: cur_step.fade_in,
                            hold: cur_step.hold,
                            runtime: rt,
                            fadein_runtime: cur_step
                                .fade_in
                                .map(|fade_in| {
                                    FadeInRuntime::new(prev_last_frame, &cur_first_frame, fade_in)
                                })
                                .map(|rt| -> Box<dyn EffectRuntime> { Box::new(rt) }),
                        })
                    })
                    .collect();

                Box::new(SequenceEffectRuntime::new(steps))
            }
        }
    }
}

/// Used in [`SequenceEffectBody`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceStep {
    hold: Duration,
    fade_in: Option<Duration>, // TODO: NoneではなくDuration::ZEROを使う。Optionを使うのはResolvedだけ
    // TODO: Template + propsもできるようにする
    body: EffectBodyOrReference<EffectBody, EffectId>,
}

/// Used in [`SequenceEffectRuntime`].
///
/// This is self-contained.
pub struct ResolvedSequenceStep {
    hold: Duration,
    fade_in: Option<Duration>,
    runtime: Box<dyn EffectRuntime>,
    fadein_runtime: Option<Box<dyn EffectRuntime>>,
}

/// `body_resolver`を使って再帰的にpropsを適用していく
fn resolve_steps<Body, Id>(
    steps: &[SequenceTemplateStepBase<Body, Id>],
    given_props: HashMap<String, Value>,
    body_resolver: impl Fn(
        &EffectBodyOrReference<Body, Id>,
        HashMap<String, Value>,
    ) -> Box<dyn EffectRuntime>,
) -> Vec<ResolvedSequenceStep> {
    steps
        .iter()
        .scan(
            // TODO: SimpleEffectRuntime::empty()みたいなやつ作る。作るときにfixturesを渡す。
            Vec::new(),
            |prev_last_frame, cur_step| {
                let hold = match &cur_step.hold {
                    Expression::Value(val) => val.unwrap_duration(),
                    Expression::Prop(p_name) => given_props.get(p_name).unwrap().unwrap_duration(),
                };

                let fade_in = match &cur_step.fade_in {
                    Expression::Value(val) => val.unwrap_duration(),
                    Expression::Prop(p_name) => given_props.get(p_name).unwrap().unwrap_duration(),
                };
                let fade_in = if fade_in == Duration::ZERO {
                    None
                } else {
                    Some(fade_in)
                };

                // TODO: そのまま渡すんじゃなくてbodyで定義されてるやつだけ渡す
                let runtime = body_resolver(&cur_step.body, given_props.clone());

                let fadein_runtime = fade_in
                    .map(|fade_in| {
                        FadeInRuntime::new(prev_last_frame, &runtime.first_frame_hint(), fade_in)
                    })
                    .map(|rt| -> Box<dyn EffectRuntime> { Box::new(rt) });

                *prev_last_frame = runtime.last_frame_hint();
                Some(ResolvedSequenceStep {
                    hold,
                    fade_in,
                    runtime,
                    fadein_runtime,
                })
            },
        )
        .collect()
}

pub struct SequenceEffectRuntime {
    /// fade_inとhold両方のruntime
    steps: Vec<ResolvedSequenceStep>,
    time_to_next_action: Duration,
    current_step_num: usize,
    running_state: SequenceStepState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequenceStepState {
    Hold,
    FadeIn,
}

impl EffectRuntime for SequenceEffectRuntime {
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand> {
        let mut commands = match self.running_state {
            SequenceStepState::FadeIn => self
                .get_current_step()
                .fadein_runtime
                .as_mut()
                .unwrap()
                .run(elapsed),
            SequenceStepState::Hold => self.get_current_step().runtime.run(elapsed),
        };

        if self.time_to_next_action >= elapsed {
            // action継続
            self.time_to_next_action -= elapsed;
            return commands;
        }

        if self.steps.len() == self.current_step_num
            && self.running_state == SequenceStepState::Hold
        {
            //全ステップ終わった
            commands.push(EffectCommand::StopEffect);
        } else if self.running_state == SequenceStepState::Hold {
            // Hold -> next step
            self.current_step_num += 1;
            let next_step = self.steps.get(self.current_step_num).unwrap();
            if let Some(fade_in) = next_step.fade_in {
                // このフレームで余った分を次のactionに追加
                self.running_state = SequenceStepState::FadeIn;
                self.time_to_next_action = fade_in - (elapsed - self.time_to_next_action);
            } else {
                self.running_state = SequenceStepState::Hold;
                self.time_to_next_action = next_step.hold - (elapsed - self.time_to_next_action);
            }
        } else {
            // FadeIn -> Hold
            self.running_state = SequenceStepState::Hold;
            self.time_to_next_action = self.steps.get(self.current_step_num).unwrap().hold
                - (elapsed - self.time_to_next_action);
        }

        commands
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        self.steps.get(0).unwrap().runtime.first_frame_hint()
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        self.steps.last().unwrap().runtime.first_frame_hint()
    }
}

impl SequenceEffectRuntime {
    pub(super) fn new(steps: Vec<ResolvedSequenceStep>) -> Self {
        let first_step = steps.get(0).unwrap();
        let (time_to_next_action, running_state) = if let Some(fade_in) = first_step.fade_in {
            (fade_in, SequenceStepState::FadeIn)
        } else {
            (first_step.hold, SequenceStepState::Hold)
        };
        Self {
            steps,
            time_to_next_action,
            current_step_num: 0,
            running_state,
        }
    }

    fn get_current_step(&mut self) -> &mut ResolvedSequenceStep {
        self.steps.get_mut(self.current_step_num).unwrap()
    }
}

struct FadeInRuntime {
    from_values: HashMap<(FixtureId, usize), u8>,
    to_values: HashMap<(FixtureId, usize), u8>,
    ordered_keys: Vec<(FixtureId, usize)>,
    duration: Duration,
    elapsed: Duration,
}

impl FadeInRuntime {
    fn new(
        from_step_last_frame: &[EffectCommand],
        to_step_first_frame: &[EffectCommand],
        duration: Duration,
    ) -> Self {
        let mut from_values = HashMap::new();
        let mut to_values = HashMap::new();
        let mut ordered_keys = Vec::new();

        for command in from_step_last_frame {
            if let EffectCommand::WriteUniverse {
                fixture_id,
                channel,
                value,
            } = command
            {
                let key = (*fixture_id, *channel);
                if !ordered_keys.contains(&key) {
                    ordered_keys.push(key);
                }
                from_values.insert(key, *value);
            }
        }
        for command in to_step_first_frame {
            if let EffectCommand::WriteUniverse {
                fixture_id,
                channel,
                value,
            } = command
            {
                let key = (*fixture_id, *channel);
                if !ordered_keys.contains(&key) {
                    ordered_keys.push(key);
                }
                to_values.insert(key, *value);
            }
        }

        Self {
            from_values,
            to_values,
            ordered_keys,
            duration,
            elapsed: Duration::ZERO,
        }
    }
}

impl EffectRuntime for FadeInRuntime {
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand> {
        self.elapsed = self.elapsed.saturating_add(elapsed);

        let ratio = if self.duration.is_zero() || self.elapsed >= self.duration {
            1.0
        } else {
            self.elapsed.as_secs_f64() / self.duration.as_secs_f64()
        };

        self.ordered_keys
            .iter()
            .map(|key| {
                let from = self.from_values.get(key).copied().unwrap_or(0) as f64;
                let to = self.to_values.get(key).copied().unwrap_or(0) as f64;
                let value = (from + (to - from) * ratio).round() as u8;
                EffectCommand::WriteUniverse {
                    fixture_id: key.0,
                    channel: key.1,
                    value,
                }
            })
            .collect()
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        panic!("first_frame_hint() should not be called on FadeInRuntime");
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        panic!("last_frame_hint() should not be called on FadeInRuntime");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_function_is_serialized_and_deserialized_correctly() {
        todo!()
        /*let fxt_id = FixtureId::new();
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
                    body: EffectBodyOrReference::Body(EffectBody::Simple(SimpleEffectBody::new(
                        vec![((fxt_id, 0usize), 255u8), ((fxt_id, 1), 200)]
                            .into_iter()
                            .collect::<HashMap<_, _>>(),
                    ))),
                },
                SequenceStep {
                    duration: Duration::from_millis(700),
                    fade_in: Duration::from_millis(100),
                    body: EffectBodyOrReference::Reference(simple.id()),
                },
            ],
        );

        let json = serde_json::to_string_pretty(&fun).unwrap();
        println!("{}", json);

        let deserialized: Effect = serde_json::from_str(&json).unwrap();

        assert_eq!(fun, deserialized);*/
    }

    #[test]
    fn fadein_works() {
        todo!()
    }
}
