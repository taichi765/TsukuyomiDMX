use crate::effects::EffectRuntime;

use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectSpecBody {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelEffectBody {
    items: Vec<EffectBodyOrReference<EffectBody, EffectId>>,
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
    fn run(&mut self, elapsed: Duration) -> Vec<EffectCommand> {
        /*let EffectBody::Parallel(this) = todo!() else {
            unreachable!()
        };
        self.runtimes
            .iter_mut()
            .zip(&this.items)
            .fold(Vec::new(), |mut acc, (rt, data)| {
                let mut commands = match data {
                    EffectBodyOrReference::Body(fun) => rt.run(elapsed, doc.clone()),
                    EffectBodyOrReference::Reference(id) => {
                        doc.with_functions(|it| rt.run(elapsed, doc.clone()))
                    }
                };
                acc.append(&mut commands);
                acc
            })*/
        todo!()
    }

    fn first_frame_hint(&self) -> Vec<EffectCommand> {
        todo!()
    }

    fn last_frame_hint(&self) -> Vec<EffectCommand> {
        todo!()
    }
}
