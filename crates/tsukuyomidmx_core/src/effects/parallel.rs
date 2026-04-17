use crate::effects::EffectRuntime;

use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectSpecBody {}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectBody {
    items: Vec<EffectBodyOrReference<EffectBody, EffectId>>,
}

pub struct StandAloneParallelEffectRuntime {
    fun_id: EffectId,
    inner: ParallelEffectRuntime,
}

pub struct ParallelEffectRuntime {
    runtimes: Vec<Box<dyn EffectRuntime>>,
}

impl ParallelEffectSpecBody {
    pub(super) fn bind_to_inner(
        &self,
        _args: impl Iterator<Item = Vec<FixtureId>>,
        _doc: DocStateView,
        _diag: &mut Diagnostics,
    ) -> Option<ParallelEffectBody> {
        todo!()
    }
}

impl ParallelEffectBody {
    pub(super) fn new(items: impl Into<Vec<EffectBodyOrReference<EffectBody, EffectId>>>) -> Self {
        Self {
            items: items.into(),
        }
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: EffectId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneEffectRuntime> {
        Box::new(StandAloneParallelEffectRuntime {
            fun_id: self_id,
            inner: ParallelEffectRuntime {
                runtimes: self
                    .items
                    .iter()
                    .map(|fun| fun.create_runtime(doc.clone()))
                    .collect(),
            },
        })
    }

    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn EffectRuntime> {
        Box::new(ParallelEffectRuntime {
            runtimes: self
                .items
                .iter()
                .map(|fun| fun.create_runtime(doc.clone()))
                .collect(),
        })
    }
}

impl EffectRuntime for ParallelEffectRuntime {
    fn run(
        &mut self,
        this: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        let EffectBody::Parallel(this) = this else {
            unreachable!()
        };
        self.runtimes
            .iter_mut()
            .zip(&this.items)
            .fold(Vec::new(), |mut acc, (rt, data)| {
                let mut commands = match data {
                    EffectBodyOrReference::Body(fun) => rt.run(&fun, elapsed, doc.clone()),
                    EffectBodyOrReference::Reference(id) => doc.with_functions(|it| {
                        let body = &it.get(&id).unwrap().body;
                        rt.run(&body, elapsed, doc.clone())
                    }),
                };
                acc.append(&mut commands);
                acc
            })
    }
}

impl StandAloneEffectRuntime for StandAloneParallelEffectRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<EffectCommand> {
        doc.with_functions(|it| {
            let body = &it.get(&self.fun_id).unwrap().body;

            self.inner.run(body, elapsed, doc.clone())
        })
    }
}

impl EffectRuntime for StandAloneParallelEffectRuntime {
    fn run(
        &mut self,
        body: &EffectBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<EffectCommand> {
        self.inner.run(body, elapsed, doc)
    }
}
