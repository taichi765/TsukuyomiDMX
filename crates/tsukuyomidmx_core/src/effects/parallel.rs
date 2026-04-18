use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectSpecBody {
    items: Vec<SpecBodyOrReference>,
}

impl ParallelEffectSpecBody {
    pub(super) fn resolve_props(
        &self,
        given_props: HashMap<String, Value>,
        doc: DocStateView,
    ) -> Box<dyn EffectRuntime> {
        Box::new(ParallelEffectRuntime {
            runtimes: self
                .items
                .iter()
                .map(|item| item.resolve_props(given_props.clone(), doc.clone()))
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelEffectTemplateBody {
    FromSpec {
        spec_id: EffectSpecId,
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
    },
    // TODO: FromTemplate{},
    New {
        props: HashMap<String, Type>,
        items: Vec<EffectBodyOrReference<EffectTemplateBody, EffectTemplateId>>,
    },
}

impl ParallelEffectTemplateBody {
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
            } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be applied");

                let resolved_spec_props: HashMap<_, _> = spec_props
                    .iter()
                    .map(|(p_name, p_val)| match p_val {
                        Expression::Prop(p) => {
                            (p_name.to_owned(), given_props.get(p).cloned().unwrap())
                        }
                        Expression::Value(val) => (p_name.to_owned(), val.clone()),
                    })
                    .collect();

                doc.with_spec(*spec_id, |body: &ParallelEffectSpecBody| {
                    body.resolve_props(resolved_spec_props, doc.clone())
                })
            }
            Self::New { props, items } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be applied");

                Box::new(ParallelEffectRuntime {
                    runtimes: items
                        .iter()
                        .map(|item| item.resolve_props(given_props.clone(), doc.clone()))
                        .collect(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelEffectBody {
    FromTemplate {
        tmpl_id: EffectTemplateId,
        tmpl_props: HashMap<String, Value>,
    },
    New {
        items: Vec<EffectBodyOrReference<EffectBody, EffectId>>,
    },
}

impl ParallelEffectBody {
    pub(super) fn new() -> Self {
        Self::New { items: Vec::new() }
    }

    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => doc.with_template(*tmpl_id, |body: &ParallelEffectTemplateBody| {
                body.resolve_props(tmpl_props.clone(), doc.clone())
            }),
            Self::New { items } => {
                let runtimes = items
                    .iter()
                    .map(|fun| fun.create_runtime(doc.clone()))
                    .collect();
                Box::new(ParallelEffectRuntime { runtimes })
            }
        }
    }
}

pub struct ParallelEffectRuntime {
    runtimes: Vec<Box<dyn EffectRuntime>>,
}

impl EffectRuntime for ParallelEffectRuntime {
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand> {
        self.runtimes.iter_mut().fold(Vec::new(), |mut acc, rt| {
            let mut commands = rt.run(elapsed);
            acc.append(&mut commands);
            acc
        })
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        self.runtimes.iter().fold(Vec::new(), |mut acc, rt| {
            acc.append(&mut rt.first_frame_hint());
            acc
        })
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        self.runtimes.iter().fold(Vec::new(), |mut acc, rt| {
            acc.append(&mut rt.last_frame_hint());
            acc
        })
    }
}

impl EffectRegistry<ParallelEffectSpecBody, ParallelEffectTemplateBody> for DocStateView {
    fn with_spec<F, R>(&self, spec_id: EffectSpecId, f: F) -> R
    where
        F: FnOnce(&ParallelEffectSpecBody) -> R,
    {
        self.with_effect_specs(|it| {
            let EffectSpecBody::Parallel(body) = &it.get(&spec_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }

    fn with_template<F, R>(&self, tmpl_id: EffectTemplateId, f: F) -> R
    where
        F: FnOnce(&ParallelEffectTemplateBody) -> R,
    {
        self.with_effect_templates(|it| {
            let EffectTemplateBody::Parallel(body) = &it.get(&tmpl_id).unwrap().body else {
                unreachable!()
            };

            f(body)
        })
    }
}

#[cfg(test)]
mod tests {}
