pub use fixtures::*;
pub use functions::*;
pub use plugins::*;
pub use universes::*;

use crate::doc::{DocEffect, state::DocState};

pub(super) trait DocCommand {
    /// 逆コマンドを返す。
    fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect);
}

mod fixtures {
    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::fixture::{Fixture, FixtureChange, FixtureId};
    use crate::prelude::{DmxAddress, UniverseId};

    pub struct AddFixtureCommand<T> {
        fixture: Fixture,
        occupied_addresses: T, // TODO: クロスユニバースを考えるとVec<(UniverseId, DmxAddress)>を受けたほうがいいか？
    }

    impl<T> AddFixtureCommand<T> {
        pub fn new(fixture: Fixture, occupied_addresses: T) -> Self {
            Self {
                fixture,
                occupied_addresses,
            }
        }
    }

    impl<T> DocCommand for AddFixtureCommand<T>
    where
        T: Iterator<Item = (UniverseId, DmxAddress)>,
    {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let id = self.fixture.id();

            // Update address index
            let fxt = self.fixture; // due to .enumerate() moves self partially
            self.occupied_addresses
                .enumerate()
                .for_each(|(offset, (u_id, adr))| {
                    let _ = state.with_address_index_mut(|it| it.insert((u_id, adr), (id, offset)));
                });

            // Add to fixtures
            state.with_fixtures_mut(|it| it.insert(id, fxt));

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
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
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
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let removed = state.with_fixtures_mut(|it| it.remove(&self.id).unwrap());
            let occupied_addresses = state.with_fixture_defs(|it| {
                it.get(removed.fixture_def())
                    .unwrap()
                    .mode(removed.fixture_mode())
                    .unwrap()
                    .occupied_addresses(removed.universe_id(), removed.address())
            });
            (
                Box::new(AddFixtureCommand::new(removed, occupied_addresses)),
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
