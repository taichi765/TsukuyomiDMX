use itertools::Itertools;

use crate::effects::simple::SimpleEffectRuntime;

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceEffectSpecBody {
    props: HashMap<String, Type>,
    steps: Vec<SequenceTemplateStep>,
}

pub enum SequenceEffectTemplateBody {
    FromSpec {
        spec_id: EffectSpecId,
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
        fixtures: FixtureQuery,
    },
    // TODO: FromTemplate(),
    New {
        props: HashMap<String, Type>,
        steps: Vec<SequenceTemplateStep>,
        fixtures: FixtureQuery, // TODO: これいる？
    },
}

impl SequenceEffectTemplateBody {
    pub fn from_spec(spec_id: EffectSpecId, doc: DocStateView) -> Result<Self, FromSpecError> {
        let spec_props = doc.with_effect_specs(|it| {
            let spec = it
                .get(&spec_id)
                .ok_or_else(|| FromSpecError::SpecNotFound(spec_id))?;
            let EffectSpecBody::Sequence(body) = &spec.body else {
                unreachable!()
            };
            let ret = body
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
            fixtures: FixtureQuery::default(),
        })
    }

    fn resolve_props(&self, given_props: HashMap<String, Value>) -> Vec<ResolvedSequenceStep> {
        match self {
            SequenceEffectTemplateBody::FromSpec {
                spec_id,
                spec_props,
                props,
                fixtures,
            } => {
                todo!()
            }
            SequenceEffectTemplateBody::New {
                props,
                steps,
                fixtures,
            } => {
                debug_assert_eq!(props.len(), given_props.len());

                steps
                    .iter()
                    .scan(
                        // TODO: SimpleEffectRuntime::empty()みたいなやつ作る。作るときにfixturesを渡す。
                        Vec::new(),
                        |prev_last_frame, cur_step| {
                            let hold = match &cur_step.hold {
                                Expression::Value(val) => val.unwrap_duration(),
                                Expression::Prop(p_name) => {
                                    given_props.get(p_name).unwrap().unwrap_duration()
                                }
                            };

                            let fade_in = match &cur_step.fade_in {
                                Expression::Value(val) => val.unwrap_duration(),
                                Expression::Prop(p_name) => {
                                    given_props.get(p_name).unwrap().unwrap_duration()
                                }
                            };
                            let fade_in = if fade_in == Duration::ZERO {
                                None
                            } else {
                                Some(fade_in)
                            };

                            // TODO: そのまま渡すんじゃなくてbodyで定義されてるやつだけ渡す
                            let runtime = cur_step.body.resolve_props(given_props.clone());

                            let fadein_runtime = fade_in
                                .map(|fade_in| {
                                    FadeInRuntime::new(
                                        prev_last_frame,
                                        &runtime.first_frame_hint(),
                                        fade_in,
                                    )
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
        }
    }
}

pub enum FromSpecError {
    SpecNotFound(EffectSpecId),
}

pub struct SequenceEffect {
    id: EffectId,
    name: String,
    body: SequenceEffectBody,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceEffectBody {
    //TODO: FromSpec(SequenceEffectTemplateBody, HashMap<String, Value>),
    FromTemplate(EffectTemplateId, HashMap<String, Value>),
    New(Vec<SequenceStep>),
}

impl SequenceEffectBody {
    pub(super) fn create_runtime(&self, doc: DocStateView) -> SequenceEffectRuntime {
        match self {
            Self::FromTemplate(tmpl_id, tmpl_props) => doc.with_effect_templates(|it| {
                let EffectTemplateBody::Sequence(tmpl) = &it.get(tmpl_id).unwrap().body else {
                    unreachable!()
                };

                let steps = tmpl.resolve_props(tmpl_props.clone());
                SequenceEffectRuntime::new(steps)
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

                SequenceEffectRuntime::new(steps)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceTemplateStep {
    /// `fade_in`を除いた時間
    hold: Expression,
    fade_in: Expression,
    body: EffectBodyOrReference<EffectTemplateBody, EffectTemplateId>,
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

impl ResolvedSequenceStep {
    fn duration(&self) -> Duration {
        self.hold + self.fade_in.unwrap_or(Duration::ZERO)
    }
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
    fn run(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand> {
        let mut commands = match self.running_state {
            SequenceStepState::FadeIn => self
                .get_current_step()
                .fadein_runtime
                .as_mut()
                .unwrap()
                .run(elapsed, doc),
            SequenceStepState::Hold => self.get_current_step().runtime.run(elapsed, doc),
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
        todo!()
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        todo!()
    }
}

impl SequenceEffectRuntime {
    fn new(steps: Vec<ResolvedSequenceStep>) -> Self {
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

struct FadeInRuntime {}

impl FadeInRuntime {
    fn new(
        from_step_last_frame: &[EffectCommand],
        to_step: &[EffectCommand],
        duration: Duration,
    ) -> Self {
        todo!()
    }
}

impl EffectRuntime for FadeInRuntime {
    fn run(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand> {
        todo!()
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
    fn sequence_function_works() {
        let spec = EffectSpec {
            id: EffectSpecId::new(),
            name: "blink".into(),
            body: EffectSpecBody::Sequence(SequenceEffectSpecBody {
                props: vec![("duration", Type::Duration), ("color", Type::Color)]
                    .into_iter()
                    .map(|(name, typ)| (name.to_string(), typ))
                    .collect(),
                steps: vec![
                    /*SequenceTemplateStep {
                        hold: Expression::Prop("duration".into()),
                        fade_in: Expression::Value(Value::Duration(Duration::ZERO)),
                        body: EffectBodyOrReference::Body(EffectSpecBody::Simple(
                            SimpleEffectSpecBody {
                                dimmer: Some(Expression::Value(Value::Dimmer(255))),
                                color: Some(Expression::Prop("color".into())),
                            },
                        )),
                    },
                    SequenceTemplateStep {
                        hold: Expression::Prop("duration".into()),
                        fade_in: Expression::Value(Value::Duration(Duration::ZERO)),
                        body: EffectBodyOrReference::Body(EffectTemplateBody::Simple(
                            SimpleEffectTemplateBody {
                                dimmer: Some(Expression::Value(Value::Dimmer(0))),
                                color: Some(Expression::Prop("color".into())),
                            },
                        )),
                    },*/
                ],
            }),
        };

        let tmpl = EffectTemplate {
            id: EffectTemplateId::new(),
            name: "red-blink-on-left".into(),
            body: EffectTemplateBody::Sequence(SequenceEffectTemplateBody::FromSpec {
                spec_id: spec.id(),
                spec_props: HashMap::from([(
                    "color".into(),
                    Expression::Value(Value::Color([255, 0, 0])),
                )]),
                props: HashMap::from([("duration".into(), Type::Duration)]),
                fixtures: FixtureQuery::from_str(".left").unwrap(),
            }),
        };

        assert_eq!(EffectTemplate::from_spec(spec.id(), ".left"), tmpl);

        let fx = Effect {
            id: EffectId::new(),
            name: "red-blink-on-left-500ms".into(),
            body: EffectBody::Sequence(SequenceEffectBody::FromTemplate(
                tmpl.id,
                HashMap::from([(
                    "duration".into(),
                    Value::Duration(Duration::from_millis(500)),
                )]),
            )),
        };

        //let rt = fx.body.create_runtime();
    }
}
