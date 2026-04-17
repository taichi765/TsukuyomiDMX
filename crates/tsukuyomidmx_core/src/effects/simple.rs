use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleEffectSpecBody {
    dimmer: Option<u8>,
    color: Option<[u8; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "SimpleEffectDto", into = "SimpleEffectDto")]
pub struct SimpleEffectBody {
    /// (id, offset) -> value
    values: HashMap<(FixtureId, usize), u8>,
}

/// スタンドアロン
pub struct StandAloneSimpleEffectRuntime {
    fun_id: EffectId,
    inner: SimpleEffectRuntime,
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

impl SimpleEffectBody {
    pub(super) fn new(values: impl Into<HashMap<(FixtureId, usize), u8>>) -> Self {
        Self {
            values: values.into(),
        }
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: EffectId,
    ) -> Box<dyn StandAloneEffectRuntime> {
        Box::new(StandAloneSimpleEffectRuntime {
            fun_id: self_id,
            inner: SimpleEffectRuntime,
        })
    }

    pub(super) fn create_runtime(&self) -> Box<dyn EffectRuntime> {
        Box::new(SimpleEffectRuntime)
    }
}

impl SimpleEffectSpecBody {
    pub(super) fn bind_to_inner(
        &self,
        mut args: impl Iterator<Item = Vec<FixtureId>>,
        doc: DocStateView,
        diag: &mut Diagnostics,
    ) -> Option<SimpleEffectBody> {
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

        Some(SimpleEffectBody { values })
    }

    pub(super) fn new(dimmer: Option<u8>, color: Option<[u8; 3]>) -> Self {
        Self { dimmer, color }
    }
}

impl EffectRuntime for SimpleEffectRuntime {
    fn run(
        &mut self,
        body: &EffectBody,
        _elapsed: Duration,
        _doc: DocStateView,
    ) -> Vec<EffectCommand> {
        let EffectBody::Simple(fun) = body else {
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

impl StandAloneEffectRuntime for StandAloneSimpleEffectRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand> {
        doc.with_functions(|it| {
            let this = &it.get(&self.fun_id).unwrap().body;
            self.inner.run(this, elapsed, doc.clone())
        })
    }
}

impl EffectRuntime for StandAloneSimpleEffectRuntime {
    fn run(
        &mut self,
        body: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        self.inner.run(body, elapsed, doc)
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
