use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    doc::DocStateView,
    effects::{
        CreateRuntime, EffectCommand, EffectId, EffectRuntime, EffectSpecId, EffectTemplateId,
        Expression, FixtureQuery, PropsResolver, Type, Value,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectSpecBody {
    items: Vec<(EffectSpecId, FixtureQuery)>,
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
                .map(|spec_key| doc.resolve_props(spec_key, given_props.clone()))
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParallelEffectTemplateBody {
    FromSpec {
        spec_key: (EffectSpecId, FixtureQuery),
        spec_props: HashMap<String, Expression>,
        props: HashMap<String, Type>,
    },
    // TODO: FromTemplate{},
    New {
        props: HashMap<String, Type>,
        items: Vec<EffectTemplateId>,
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
                spec_key,
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

                doc.resolve_props(spec_key, resolved_spec_props)
            }
            Self::New { props, items } => {
                debug_assert_eq!(props.len(), given_props.len(), "all props must be applied");

                Box::new(ParallelEffectRuntime {
                    runtimes: items
                        .iter()
                        .map(|tmpl_id| doc.resolve_props(*tmpl_id, given_props.clone()))
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
        items: Vec<EffectId>,
    },
}

impl ParallelEffectBody {
    pub(super) fn new() -> Self {
        Self::New { items: Vec::new() }
    }

    pub(super) fn create_runtime(
        &self,
        doc: impl PropsResolver<EffectTemplateId> + CreateRuntime,
    ) -> Box<dyn EffectRuntime> {
        match self {
            Self::FromTemplate {
                tmpl_id,
                tmpl_props,
            } => doc.resolve_props(*tmpl_id, tmpl_props.to_owned()),
            Self::New { items } => {
                let runtimes = items
                    .iter()
                    .map(|effect_id| doc.create_runtime(*effect_id))
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

/*impl EffectRegistry<ParallelEffectSpecBody, ParallelEffectTemplateBody> for DocStateView {
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
}*/

#[cfg(test)]
mod tests {}
