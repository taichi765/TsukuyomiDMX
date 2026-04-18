use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleEffectSpecBody {
    pub dimmer: Option<Expression>,
    pub color: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleEffectTemplateBody {
    FromSpec {
        spec_id: EffectSpecId,
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
        fixtures: FixtureQuery,
    },
    // TODO: FromTemplate(),
    New {
        values: HashMap<(FixtureId, usize), u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimpleEffectBody {
    // TODO: FromSpec(),
    FromTemplate(EffectTemplateId, HashMap<String, Expression>),
    New {
        fixtures: FixtureQuery,
        values: HashMap<(FixtureId, usize), u8>,
    },
}

pub struct SimpleEffectRuntime;

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

pub enum SimpleTemplateOrEffect {
    Template(SimpleEffectTemplateBody),
    Effect(SimpleEffectBody),
}

impl SimpleEffectBody {
    pub(super) fn new(values: impl Into<HashMap<(FixtureId, usize), u8>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    pub(super) fn create_runtime(&self) -> Box<dyn EffectRuntime> {
        Box::new(SimpleEffectRuntime)
    }
}

impl SimpleEffectSpecBody {
    pub(super) fn new() -> Self {
        Self {
            dimmer: None,
            color: None,
        }
    }
}

impl SimpleEffectTemplateBody {
    /*#[must_use]
    pub(super) fn apply_prop(&self, name: String, value: Value) -> SimpleTemplateOrEffect {
        if self.props.len() == 1 {
            SimpleTemplateOrEffect::Effect(SimpleEffectBody { values: () })
        } else {
            SimpleTemplateOrEffect::Template(SimpleEffectTemplateBody {
                spec: self.spec,
                template: todo!(),
                props: (),
                values: (),
            })
        }
    }*/
}

impl EffectRuntime for SimpleEffectRuntime {
    fn run(&mut self, _elapsed: Duration, _doc: DocStateView) -> Vec<EffectCommand> {
        let EffectBody::Simple(fun) = todo!() else {
            unreachable!()
        };
        fun.values
            .iter()
            .fold(Vec::new(), |mut acc, ((fxt_id, offset), val)| {
                acc.push(EffectCommand::WriteUniverse {
                    fixture_id: *fxt_id,
                    channel: *offset,
                    value: *val,
                });
                acc
            })
    }
}

impl From<SimpleEffectBody> for SimpleEffectDto {
    fn from(value: SimpleEffectBody) -> Self {
        Self {
            values: value
                .values
                .into_iter()
                .map(|((fxt_id, offset), value)| SimpleEffectValueDto {
                    fxt_id: fxt_id,
                    offset: offset,
                    value: value,
                })
                .collect(),
        }
    }
}

impl From<SimpleEffectDto> for SimpleEffectBody {
    fn from(value: SimpleEffectDto) -> Self {
        Self {
            values: value
                .values
                .into_iter()
                .map(|v| ((v.fxt_id, v.offset), v.value))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_function_is_serialized_and_deserialized_correctly() {
        let fxt_id = FixtureId::new();
        let fun = Effect::new_simple(
            "Scene 1",
            vec![((fxt_id, 0usize), 255u8), ((fxt_id, 1), 200)]
                .into_iter()
                .collect::<HashMap<_, _>>(),
        );

        let json = serde_json::to_string(&fun).unwrap();

        let deserialized: Effect = serde_json::from_str(&json).unwrap();

        assert_eq!(fun, deserialized);
    }
}
