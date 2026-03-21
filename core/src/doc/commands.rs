use std::any::Any;

pub use fixtures::*;
pub use functions::*;
pub use plugins::*;
pub use universes::*;

use crate::doc::{DocEffect, state::DocState};

pub(super) trait DocCommand {
    /// 逆コマンドを返す。
    #[must_use]
    fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect);

    /// This allows downcasting to specific command types so that you can use `assert_eq!()`
    #[allow(dead_code)] // Actually this function is used in tests, but cargo doesn't recognize it
    fn as_any(&self) -> &dyn Any;
}

mod fixtures {
    use std::fmt::Debug;

    use derive_getters::Getters;

    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::fixture::{Fixture, FixtureChange, FixtureId};
    use crate::prelude::{DmxAddress, UniverseId};

    #[derive(Debug, Clone, PartialEq, Getters)]
    pub struct AddFixtureCommand<T> {
        fixture: Fixture,
        occupied_addresses: T,
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
        T: Iterator<Item = (UniverseId, DmxAddress)> + 'static,
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

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct UpdateFixtureCommand<T> {
        id: FixtureId,
        change: FixtureChange,
        old_occupied_addresses: T,
        new_occupied_addresses: T,
    }

    impl<T> UpdateFixtureCommand<T> {
        pub fn new(
            id: FixtureId,
            change: FixtureChange,
            old_occupied_addresses: T,
            new_occupied_addresses: T,
        ) -> Self {
            Self {
                id,
                change,
                old_occupied_addresses,
                new_occupied_addresses,
            }
        }
    }

    impl<T> DocCommand for UpdateFixtureCommand<T>
    where
        T: Iterator<Item = (UniverseId, DmxAddress)> + Clone + 'static,
    {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let rev_change = state.with_fixtures(|it| {
                let fxt = it.get(&self.id).unwrap();
                self.change.inverse_from(fxt)
            });

            self.old_occupied_addresses.clone().for_each(|(uni, adr)| {
                let _ = state.with_address_index_mut(|index| index.remove(&(uni, adr)));
            });
            self.new_occupied_addresses
                .clone()
                .enumerate()
                .for_each(|(offset, (uni, adr))| {
                    let _ = state.with_address_index_mut(|index| {
                        index.insert((uni, adr), (self.id, offset))
                    });
                });

            let (id, change) = (self.id, self.change);
            state.with_fixtures_mut(|it| {
                it.get_mut(&id).unwrap().apply_change(change);
            });

            (
                Box::new(UpdateFixtureCommand::new(
                    id,
                    rev_change,
                    self.new_occupied_addresses,
                    self.old_occupied_addresses,
                )),
                DocEffect::FixtureUpdated(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, PartialEq, Clone)]
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
            occupied_addresses.clone().for_each(|(uni, adr)| {
                let _ = state.with_address_index_mut(|index| index.remove(&(uni, adr)));
            });
            (
                Box::new(AddFixtureCommand::new(removed, occupied_addresses)),
                DocEffect::FixtureRemoved(self.id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
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
