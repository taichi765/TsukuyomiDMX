use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleEffectSpecBody {
    pub dimmer: Option<Expression>,
    pub color: Option<Expression>,
}

impl SimpleEffectSpecBody {
    pub(super) fn new() -> Self {
        Self {
            dimmer: None,
            color: None,
        }
    }

    pub(super) fn resolve_props(
        &self,
        fixtures: &FixtureQuery,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        let fixtures = fixtures.query(doc.clone());
        debug_assert_ne!(fixtures.len(), 0);

        let values = fixtures.iter().fold(HashMap::new(), |mut acc, fxt_id| {
            let (dimmer_channel, color_channel) = doc.get_dimmer_and_color(*fxt_id);

            if let Some(val) = &self.dimmer {
                let val = match val {
                    Expression::Value(val) => val.clone(),
                    Expression::Prop(p_name) => given_props.get(p_name).cloned().unwrap(),
                };
                acc.insert((*fxt_id, dimmer_channel.unwrap()), val.unwrap_dimmer());
            }

            if let Some(val) = &self.color {
                let val = match val {
                    Expression::Value(val) => val.clone(),
                    Expression::Prop(p_name) => given_props.get(p_name).cloned().unwrap(),
                };
                for (i, ch) in color_channel.unwrap().iter().enumerate() {
                    acc.insert((*fxt_id, *ch), val.unwrap_color()[i]);
                }
            }
            acc
        });

        Box::new(SimpleEffectRuntime::new(values))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimpleEffectTemplateBody {
    FromSpec {
        spec_id: EffectSpecId,
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
        fixtures: FixtureQuery,
    },
    // TODO: FromTemplate(),
    New {
        fixtures: FixtureQuery,
        values: HashMap<(FixtureId, usize), u8>,
        dimmer: Expression,
        color: Expression,
        props: HashMap<String, Type>,
    },
}

impl SimpleEffectTemplateBody {
    pub fn new() -> Self {
        Self::New {
            fixtures: FixtureQuery::default(),
            values: HashMap::new(),
            dimmer: Expression::Value(Value::Dimmer(0)),
            color: Expression::Value(Value::Color([0, 0, 0])),
            props: HashMap::new(),
        }
    }

    pub(super) fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromSpec {
                spec_id,
                spec_props,
                props,
                fixtures,
            } => {
                let resolved_spec_props = spec_props
                    .iter()
                    .map(|(p_name, p_val)| match p_val {
                        Expression::Prop(p) => {
                            (p_name.to_owned(), given_props.get(p).cloned().unwrap())
                        }
                        Expression::Value(val) => (p_name.to_owned(), val.clone()),
                    })
                    .collect();
                doc.with_spec(*spec_id, |body: &SimpleEffectSpecBody| {
                    body.resolve_props(fixtures, resolved_spec_props, doc.clone())
                })
            }
            Self::New {
                fixtures,
                values,
                dimmer,
                color,
                props,
            } => {
                let dimmer = match dimmer {
                    Expression::Value(val) => val.unwrap_dimmer(),
                    Expression::Prop(p_name) => given_props.get(p_name).unwrap().unwrap_dimmer(),
                };

                let color = match color {
                    Expression::Value(val) => val.unwrap_color(),
                    Expression::Prop(p_name) => given_props.get(p_name).unwrap().unwrap_color(),
                };

                let fixtures = fixtures.query(doc.clone());
                let values = fixtures.iter().fold(values.clone(), |mut acc, fxt_id| {
                    let (dimmer_ch, color_ch) = doc.clone().get_dimmer_and_color(*fxt_id);
                    acc.insert((*fxt_id, dimmer_ch.unwrap()), dimmer);
                    color.iter().zip(color_ch.unwrap()).for_each(|(val, ch)| {
                        acc.insert((*fxt_id, ch), *val);
                    });
                    acc
                });
                Box::new(SimpleEffectRuntime::new(values))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    from = "SimpleEffectBodyDto",
    into = "SimpleEffectBodyDto",
    deny_unknown_fields
)]
pub enum SimpleEffectBody {
    // TODO: FromSpec(),
    FromTemplate {
        tmpl_id: EffectTemplateId,
        tmpl_props: HashMap<String, Value>,
    },
    New {
        fixtures: FixtureQuery,
        values: HashMap<(FixtureId, usize), u8>,
    },
}

impl SimpleEffectBody {
    pub(super) fn new() -> Self {
        Self::New {
            fixtures: FixtureQuery::default(),
            values: HashMap::new(),
        }
    }

    pub(super) fn create_runtime(
        &self,
        doc: impl PropsResolver<EffectTemplateId>,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => doc.resolve_props(*tmpl_id, tmpl_props.clone()),
            Self::New { fixtures, values } => Box::new(SimpleEffectRuntime::new(values.clone())),
        }
    }
}

pub struct SimpleEffectRuntime {
    commands: Vec<EffectCommand>,
}

impl EffectRuntime for SimpleEffectRuntime {
    fn run(&mut self, _elapsed: Duration) -> Vec<EffectCommand> {
        self.commands.clone()
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        self.commands.clone()
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        self.commands.clone()
    }
}

impl SimpleEffectRuntime {
    pub(super) fn new(values: HashMap<(FixtureId, usize), u8>) -> Self {
        Self {
            commands: values
                .iter()
                .map(|((fxt_id, offset), val)| EffectCommand::WriteUniverse {
                    fixture_id: *fxt_id,
                    channel: *offset,
                    value: *val,
                })
                .collect(),
        }
    }
}

/// DTO for [`SimpleEffectBody`].
///
/// DTO is requied because the key type of `HashMap` in [`SimpleEffectBody`] is not string.
#[derive(Serialize, Deserialize)]
enum SimpleEffectBodyDto {
    FromTemplate {
        tmpl_id: EffectTemplateId,
        tmpl_props: HashMap<String, Value>,
    },
    New {
        fixtures: FixtureQuery,
        values: Vec<(FixtureId, usize, u8)>,
    },
}

impl From<SimpleEffectBody> for SimpleEffectBodyDto {
    fn from(value: SimpleEffectBody) -> Self {
        match value {
            SimpleEffectBody::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            },
            SimpleEffectBody::New { fixtures, values } => Self::New {
                fixtures,
                values: values
                    .into_iter()
                    .map(|((fxt_id, offset), val)| (fxt_id, offset, val))
                    .collect(),
            },
        }
    }
}

impl From<SimpleEffectBodyDto> for SimpleEffectBody {
    fn from(value: SimpleEffectBodyDto) -> Self {
        match value {
            SimpleEffectBodyDto::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            },
            SimpleEffectBodyDto::New { fixtures, values } => Self::New {
                fixtures,
                values: values
                    .into_iter()
                    .map(|(fxt_id, offset, val)| ((fxt_id, offset), val))
                    .collect(),
            },
        }
    }
}

impl EffectRegistry<SimpleEffectSpecBody, SimpleEffectTemplateBody> for DocStateView {
    fn with_spec<F, R>(&self, spec_id: EffectSpecId, f: F) -> R
    where
        F: FnOnce(&SimpleEffectSpecBody) -> R,
    {
        self.with_effect_specs(|it| {
            let EffectSpecBody::Simple(body) = &it.get(&spec_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }

    fn with_template<F, R>(&self, tmpl_id: EffectTemplateId, f: F) -> R
    where
        F: FnOnce(&SimpleEffectTemplateBody) -> R,
    {
        self.with_effect_templates(|it| {
            let EffectTemplateBody::Simple(body) = &it.get(&tmpl_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }
}

trait DocStateViewExt {
    fn get_dimmer_and_color(&self, fxt_id: FixtureId) -> (Option<usize>, Option<[usize; 3]>);
}

impl DocStateViewExt for DocStateView {
    /// Get the offset of dimmer channel and color channel in the fixture.
    fn get_dimmer_and_color(&self, fxt_id: FixtureId) -> (Option<usize>, Option<[usize; 3]>) {
        self.with_fixtures_and_defs(|fxts, defs| {
            let fxt = fxts.get(&fxt_id).unwrap();
            let def = defs.get(fxt.fixture_def()).unwrap();
            let dimmer_channel = def.find_dimmer_channel_in_mode(fxt.fixture_mode());
            let color_channel = def.find_rgb_channel_in_mode(fxt.fixture_mode());
            (dimmer_channel, color_channel)
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Panics when [`PropsResolver::resolve_props()`] is called.
    struct DummyPropsResolver;

    impl PropsResolver<EffectTemplateId> for DummyPropsResolver {
        fn resolve_props(
            &self,
            id: EffectTemplateId,
            given_props: HashMap<String, Value>,
            // fixtures: &FixtureQuery,
        ) -> Box<dyn EffectRuntime> {
            unimplemented!("This is dummy!")
        }
    }

    fn create_simple_effect_with_some_values() -> (Effect, FixtureId) {
        let fxt_id = FixtureId::new();
        let mut effect = Effect::new_simple("Scene 1");

        let EffectBody::Simple(SimpleEffectBody::New { fixtures, values }) = effect.body() else {
            panic!("should match")
        };

        let new_values = HashMap::from([((fxt_id, 0), 255), ((fxt_id, 1), 200)]);
        let new = SimpleEffectBody::New {
            fixtures: fixtures.clone(),
            values: new_values,
        };
        effect.apply_change(EffectChange::Simple(new));
        (effect, fxt_id)
    }

    #[test]
    fn simple_effect_is_serialized_and_deserialized_correctly() {
        let (effect, fxt_id) = create_simple_effect_with_some_values();

        let json = serde_json::to_string_pretty(&effect).unwrap();

        let deserialized: Effect = serde_json::from_str(&json).unwrap();

        assert_eq!(effect, deserialized);
    }

    #[test]
    fn simple_runtime_run_returns_right_command() {
        let (effect, fxt_id) = create_simple_effect_with_some_values();

        let mut rt = if let EffectBody::Simple(body) = effect.body() {
            body.create_runtime(DummyPropsResolver)
        } else {
            panic!("should match")
        };

        let commands = rt.run(Duration::from_millis(20));

        assert_eq!(commands.len(), 2);
        assert!(commands.iter().any(|cmd| match cmd {
            EffectCommand::WriteUniverse {
                fixture_id,
                channel,
                value,
            } if *fixture_id == fxt_id && *channel == 0 && *value == 255 => true,
            _ => false,
        }));

        assert!(commands.iter().any(|cmd: &EffectCommand| match cmd {
            EffectCommand::WriteUniverse {
                fixture_id,
                channel,
                value,
            } if (*fixture_id == fxt_id && *channel == 1 && *value == 200) => true,
            _ => false,
        }));
    }

    #[test]
    fn simple_runtime_returns_same_commands_between_first_and_last_frame() {
        let (effect, _) = create_simple_effect_with_some_values();

        let rt = if let EffectBody::Simple(body) = effect.body() {
            body.create_runtime(DummyPropsResolver)
        } else {
            panic!("should match")
        };

        assert_eq!(rt.first_frame_hint(), rt.last_frame_hint());
    }
}
