use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleEffectSpecBody {
    dimmer: Option<Expression>,
    color: Option<Expression>,
}

impl SimpleEffectSpecBody {
    pub(super) fn new() -> Self {
        Self {
            dimmer: None,
            color: None,
        }
    }

    fn resolve_props(
        &self,
        fixtures: &FixtureQuery,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> HashMap<(FixtureId, usize), u8> {
        let fixtures = fixtures.query(doc.clone());
        debug_assert_ne!(fixtures.len(), 0);

        fixtures.iter().fold(HashMap::new(), |mut acc, fxt_id| {
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
        })
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
    fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        rg: impl EffectRegistry<SimpleEffectSpecBody, SimpleEffectTemplateBody>,
        doc: DocStateView,
    ) -> HashMap<(FixtureId, usize), u8> {
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
                rg.with_spec(*spec_id, |body| {
                    body.resolve_props(fixtures, resolved_spec_props, doc)
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
                fixtures.iter().fold(values.clone(), |mut acc, fxt_id| {
                    let (dimmer_ch, color_ch) = doc.clone().get_dimmer_and_color(*fxt_id);
                    acc.insert((*fxt_id, dimmer_ch.unwrap()), dimmer);
                    color.iter().zip(color_ch.unwrap()).for_each(|(val, ch)| {
                        acc.insert((*fxt_id, ch), *val);
                    });
                    acc
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        rg: impl EffectRegistry<SimpleEffectSpecBody, SimpleEffectTemplateBody> + Clone,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => {
                let resolved_value = rg.with_template(*tmpl_id, |body| {
                    body.resolve_props(tmpl_props.clone(), rg.clone(), doc)
                });
                Box::new(SimpleEffectRuntime::new(resolved_value))
            }
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

/// DTO for [`SimpleEffect`].
///
/// DTO is requied because the key type of `HashMap` in [`SimpleEffect`] is not string.
#[derive(Serialize, Deserialize)]
struct SimpleEffectDto {
    values: Vec<SimpleEffectValueDto>,
}

#[derive(Serialize, Deserialize)]
struct SimpleEffectValueDto {
    fxt_id: FixtureId,
    offset: usize,
    value: u8,
}

impl From<SimpleEffectBody> for SimpleEffectDto {
    fn from(value: SimpleEffectBody) -> Self {
        /*Self {
            values: value
                .values
                .into_iter()
                .map(|((fxt_id, offset), value)| SimpleEffectValueDto {
                    fxt_id: fxt_id,
                    offset: offset,
                    value: value,
                })
                .collect(),
        }*/
        todo!()
    }
}

impl From<SimpleEffectDto> for SimpleEffectBody {
    fn from(value: SimpleEffectDto) -> Self {
        /*Self {
            values: value
                .values
                .into_iter()
                .map(|v| ((v.fxt_id, v.offset), v.value))
                .collect(),
        }*/
        todo!()
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

    #[test]
    fn simple_function_is_serialized_and_deserialized_correctly() {
        let fxt_id = FixtureId::new();
        let fun = Effect::new_simple("Scene 1");

        // TODO: Add values here

        let json = serde_json::to_string(&fun).unwrap();

        let deserialized: Effect = serde_json::from_str(&json).unwrap();

        assert_eq!(fun, deserialized);
    }
}
