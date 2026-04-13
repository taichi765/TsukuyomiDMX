use crate::functions::FunctionRuntime;

use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelFunctionPrototypeBody {}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelFunctionBody {}

pub struct StandAloneParallelFunctionRuntime {
    fun_id: AppliedFunctionId,
    inner: ParallelFunctionRuntime,
}

pub struct ParallelFunctionRuntime {}

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
    pub(super) fn new() -> Self {
        todo!()
    }

    pub(super) fn create_runtime_standalone(
        &self,
        self_id: AppliedFunctionId,
    ) -> Box<dyn StandAloneFunctionRuntime> {
        Box::new(StandAloneParallelFunctionRuntime {
            fun_id: self_id,
            inner: ParallelFunctionRuntime {},
        })
    }

    pub(super) fn create_runtime(&self) -> Box<dyn FunctionRuntime> {
        todo!()
    }
}

impl FunctionRuntime for ParallelFunctionRuntime {
    fn run(
        &mut self,
        _this: &FunctionBody,
        _elapsed: Duration,
        _doc: DocStateView,
    ) -> Vec<FunctionCommand> {
        todo!()
    }
}

impl StandAloneFunctionRuntime for StandAloneParallelFunctionRuntime {
    fn run_standalone(&mut self, elapsed: Duration, doc: DocStateView) -> Vec<FunctionCommand> {
        todo!()
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
