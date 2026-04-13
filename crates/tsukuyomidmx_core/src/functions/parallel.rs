use crate::functions::FunctionRuntime;

use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelFunctionPrototypeBody {}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelFunctionBody {
    items: Vec<FunctionBodyOrId<FunctionBody, AppliedFunctionId>>,
}

pub struct StandAloneParallelFunctionRuntime {
    fun_id: AppliedFunctionId,
    inner: ParallelFunctionRuntime,
}

pub struct ParallelFunctionRuntime {
    runtimes: Vec<Box<dyn FunctionRuntime>>,
}

impl ParallelFunctionPrototypeBody {
    pub(super) fn bind_to_inner(
        &self,
        _args: impl Iterator<Item = Vec<FixtureId>>,
        _doc: DocStateView,
        _diag: &mut Diagnostics,
    ) -> Option<ParallelFunctionBody> {
        todo!()
    }
}

impl ParallelFunctionBody {
    pub(super) fn new(
        items: impl Into<Vec<FunctionBodyOrId<FunctionBody, AppliedFunctionId>>>,
    ) -> Self {
        Self {
            items: items.into(),
        }
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: AppliedFunctionId,
        doc: DocStateView,
    ) -> Box<dyn StandAloneFunctionRuntime> {
        Box::new(StandAloneParallelFunctionRuntime {
            fun_id: self_id,
            inner: ParallelFunctionRuntime {
                runtimes: self
                    .items
                    .iter()
                    .map(|fun| fun.create_runtime(doc.clone()))
                    .collect(),
            },
        })
    }

    pub(super) fn create_runtime(&self, doc: DocStateView) -> Box<dyn FunctionRuntime> {
        Box::new(ParallelFunctionRuntime {
            runtimes: self
                .items
                .iter()
                .map(|fun| fun.create_runtime(doc.clone()))
                .collect(),
        })
    }
}

impl FunctionRuntime for ParallelFunctionRuntime {
    fn run(
        &mut self,
        this: &FunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        let FunctionBody::Parallel(this) = this else {
            unreachable!()
        };
        self.runtimes
            .iter_mut()
            .zip(&this.items)
            .fold(Vec::new(), |mut acc, (rt, data)| {
                let mut commands = match data {
                    FunctionBodyOrId::Body(fun) => rt.run(&fun, elapsed, doc.clone()),
                    FunctionBodyOrId::Id(id) => doc.with_functions(|it| {
                        let body = &it.get(&id).unwrap().body;
                        rt.run(&body, elapsed, doc.clone())
                    }),
                };
                acc.append(&mut commands);
                acc
            })
    }
}

impl StandAloneFunctionRuntime for StandAloneParallelFunctionRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<FunctionCommand> {
        doc.with_functions(|it| {
            let body = &it.get(&self.fun_id).unwrap().body;

            self.inner.run(body, elapsed, doc.clone())
        })
    }
}

impl FunctionRuntime for StandAloneParallelFunctionRuntime {
    fn run(
        &mut self,
        body: &FunctionBody,
        elapsed: Duration,
        doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        self.inner.run(body, elapsed, doc)
    }
}
