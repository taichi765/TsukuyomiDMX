pub use fixtures::*;
pub use functions::*;
pub use plugins::*;
pub use universes::*;

use crate::doc::{state::DocState, DocEffect};

pub trait DocCommand {
    /// 逆コマンドを返す。
    fn apply(self, state: &DocState) -> (Box<dyn DocCommand>, DocEffect);
}

mod fixtures {
    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::fixture::{Fixture, FixtureChange, FixtureId};

    pub struct AddFixtureCommand {
        fixture: Fixture,
    }

    impl AddFixtureCommand {
        pub fn new(fixture: Fixture) -> Self {
            Self { fixture }
        }
    }

    impl DocCommand for AddFixtureCommand {
        fn apply(self, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let id = self.fixture.id();
            state.with_fixtures_mut(|it| it.insert(id, self.fixture));
            (
                Box::new(RemoveFixtureCommand::new(id)),
                DocEffect::FixtureAdded(id),
            )
        }
    }

    pub struct UpdateFixtureCommand {
        id: FixtureId,
        change: FixtureChange,
    }

    impl UpdateFixtureCommand {
        pub fn new(id: FixtureId, change: FixtureChange) -> Self {
            Self { id, change }
        }
    }

    impl DocCommand for UpdateFixtureCommand {
        fn apply(self, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let rev_change = state.with_fixtures(|it| {
                let fxt = it.get(&self.id).unwrap();
                self.change.inverse_from(fxt)
            });

            let id = self.id;
            state.with_fixtures_mut(|it| {
                it.get_mut(&self.id).unwrap().apply_change(self.change);
            });
            (
                Box::new(UpdateFixtureCommand::new(id, rev_change)),
                DocEffect::FixtureUpdated(id),
            )
        }
    }

    pub struct RemoveFixtureCommand {
        id: FixtureId,
    }

    impl RemoveFixtureCommand {
        pub fn new(id: FixtureId) -> Self {
            Self { id }
        }
    }

    impl DocCommand for RemoveFixtureCommand {
        fn apply(self, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let removed = state.with_fixtures_mut(|it| it.remove(&self.id).unwrap());
            (
                Box::new(AddFixtureCommand::new(removed)),
                DocEffect::FixtureRemoved(self.id),
            )
        }
    }
}

mod functions {
    pub struct AddFunctionCommand {}

    pub struct UpdateFunctionCommand {}

    pub struct RemoveFunctionCommand {}
}

mod plugins {
    pub struct AddOutputCommand {}

    pub struct RemoveOutputCommand {}
}

mod universes {
    pub struct AddUniverseCommand {}

    pub struct RemoveUniverseCommand {}
}
