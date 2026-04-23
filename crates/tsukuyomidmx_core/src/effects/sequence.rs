use std::ops::ControlFlow;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceTemplateStepBase<T> {
    /// `fade_in`を除いた時間
    hold: Expression,
    fade_in: Expression,
    /// idまたはそれに付随する情報
    key: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceEffectSpecBody {
    props: HashMap<String, Type>,
    steps: Vec<SequenceSpecStep>,
}

type SequenceSpecStep = SequenceTemplateStepBase<(EffectSpecId, FixtureQuery)>;

impl SequenceEffectSpecBody {
    pub(super) fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        resolve_steps(&self.steps, given_props, |key, props| {
            doc.resolve_props(key, props)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceEffectTemplateBody {
    FromSpec {
        spec_key: (EffectSpecId, FixtureQuery),
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

type SequenceTemplateStep = SequenceTemplateStepBase<EffectTemplateId>;

impl SequenceEffectTemplateBody {
    /*pub fn from_spec(
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
            spec_key: spec_id,
            spec_props,
            props: HashMap::new(),
        })
    }*/

    pub(super) fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromSpec {
                spec_key,
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

                doc.resolve_props(spec_key, resolved_spec_props)
            }
            Self::New {
                props,
                steps,
                fixtures,
            } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be applied");

                resolve_steps(steps, given_props, |tmpl_id, props| {
                    doc.resolve_props(*tmpl_id, props)
                })
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
    pub(super) fn create_runtime(
        &self,
        doc: impl PropsResolver<EffectTemplateId> + CreateRuntime,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate(tmpl_id, tmpl_props) => {
                doc.resolve_props(*tmpl_id, tmpl_props.clone())
            }
            Self::New(steps) => {
                // TODO: SequenceEffectTemplateBody::resolve_props()と重複しているコードがある
                // TODO: SimpleEffectRuntime::empty()みたいなやつ作る。作るときにfixturesを渡す。
                let steps = steps
                    .iter()
                    .scan(Vec::new(), |prev_last_frame, cur_step| {
                        let rt = doc.create_runtime(cur_step.effect_id);
                        let cur_first_frame = rt.first_frame_hint();
                        *prev_last_frame = rt.last_frame_hint();
                        Some(StepRuntime::new(
                            cur_step.hold,
                            rt,
                            cur_step.fade_in,
                            cur_step
                                .fade_in
                                .map(|fade_in| {
                                    FadeInRuntime::new(prev_last_frame, &cur_first_frame, fade_in)
                                })
                                .map(|rt| -> Box<dyn EffectRuntime> { Box::new(rt) }),
                        ))
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
    effect_id: EffectId,
}

/// 再帰的にpropsを適用していく
///
/// - `item_resolver`: 各Stepのeffect_idをresolveする
fn resolve_steps<T: ResolveStepItemKey>(
    steps: &[SequenceTemplateStepBase<T>],
    given_props: HashMap<String, Value>,
    item_resolver: impl Fn(&T, HashMap<String, Value>) -> Box<dyn EffectRuntime>,
) -> Box<dyn EffectRuntime> {
    let steps = steps
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
                let runtime = item_resolver(&cur_step.key, given_props.clone());

                let fadein_runtime = fade_in
                    .map(|fade_in| {
                        FadeInRuntime::new(prev_last_frame, &runtime.first_frame_hint(), fade_in)
                    })
                    .map(|rt| -> Box<dyn EffectRuntime> { Box::new(rt) });

                *prev_last_frame = runtime.last_frame_hint();
                Some(StepRuntime::new(hold, runtime, fade_in, fadein_runtime))
            },
        )
        .collect();
    Box::new(SequenceEffectRuntime::new(steps))
}

/// [`resolve_props()`]でstepをresolveする際にkeyとして使える型
trait ResolveStepItemKey {}

impl ResolveStepItemKey for EffectTemplateId {}

impl ResolveStepItemKey for (EffectSpecId, FixtureQuery) {}

pub struct SequenceEffectRuntime {
    /// fade_inとhold両方のruntime
    steps: Vec<StepRuntime>,
    current_step_num: usize,
    /// stepをまたぐときに溢れたDuration
    dur_buffer: Duration,
}

impl EffectRuntime for SequenceEffectRuntime {
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand> {
        let (mut commands, flow) = self.get_current_step().run(elapsed);

        if let ControlFlow::Break(dur) = flow {
            self.current_step_num += 1;
            if self.current_step_num == self.steps.len() {
                commands.push(EffectCommand::StopEffect);
            } else {
                self.dur_buffer = dur;
            }
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
    fn new(steps: Vec<StepRuntime>) -> Self {
        Self {
            steps,
            current_step_num: 0,
            dur_buffer: Duration::ZERO,
        }
    }

    fn get_current_step(&mut self) -> &mut StepRuntime {
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

struct StepRuntime {
    hold: Duration,
    fade_in: Option<Duration>,
    runtime: Box<dyn EffectRuntime>,
    fadein_runtime: Option<Box<dyn EffectRuntime>>,
    time_to_next_action: Duration,
    running_state: SequenceStepState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequenceStepState {
    Hold,
    FadeIn,
}

impl StepRuntime {
    fn new(
        hold: Duration,
        runtime: Box<dyn EffectRuntime>,
        fade_in: Option<Duration>,
        fadein_runtime: Option<Box<dyn EffectRuntime>>,
    ) -> Self {
        let (time_to_next_action, running_state) = if let Some(fade_in) = fade_in {
            (fade_in, SequenceStepState::FadeIn)
        } else {
            (hold, SequenceStepState::Hold)
        };
        Self {
            hold,
            fade_in,
            runtime,
            fadein_runtime,
            time_to_next_action,
            running_state,
        }
    }

    /// 2つ目の返り値がBreak(dur)の場合、このステップでdur分だけ余って次のstepに進む
    /// Continueの場合このstepを継続
    fn run(&mut self, elapsed: Duration) -> (Vec<EffectCommand>, ControlFlow<Duration, ()>) {
        let commands = match self.running_state {
            SequenceStepState::FadeIn => self.fadein_runtime.as_mut().unwrap().run(elapsed),
            SequenceStepState::Hold => self.runtime.run(elapsed),
        };

        if self.time_to_next_action >= elapsed {
            // action継続
            self.time_to_next_action -= elapsed;
            return (commands, ControlFlow::Continue(()));
        }

        if self.running_state == SequenceStepState::Hold {
            (
                commands,
                ControlFlow::Break(elapsed - self.time_to_next_action),
            )
        } else {
            // FadeIn -> Hold
            self.running_state = SequenceStepState::Hold;
            self.time_to_next_action = self.hold - (elapsed - self.time_to_next_action);
            (commands, ControlFlow::Continue(()))
        }
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        if self.fade_in.is_none() {
            self.runtime.first_frame_hint()
        } else {
            panic!("first_frame_hint() should not be called on FadeInRuntime")
        }
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        self.runtime.last_frame_hint()
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
