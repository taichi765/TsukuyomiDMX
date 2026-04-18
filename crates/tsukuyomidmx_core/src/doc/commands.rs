use std::{any::Any, fmt::Debug};

pub use effect_specs::*;
pub use effect_templates::*;
pub use effects::*;
pub use fixtures::*;
pub use universes::*;

use crate::doc::{DocEffect, state::DocState};

pub(super) trait DocCommand: Debug {
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
        T: Iterator<Item = (UniverseId, DmxAddress)> + Debug + 'static,
    {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let id = self.fixture.id();

            // Update address index
            let fxt = self.fixture; // due to .enumerate() moves self partially

            self.occupied_addresses
                .enumerate()
                .for_each(|(offset, (u_id, adr))| {
                    let _ = state.with_address_index_mut(|it| it.insert(u_id, adr, id, offset));
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
        T: Iterator<Item = (UniverseId, DmxAddress)> + Clone + Debug + 'static,
    {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand + 'static>, DocEffect) {
            let rev_change = state.with_fixtures(|it| {
                let fxt = it.get(&self.id).unwrap();
                self.change.inverse_from(fxt)
            });

            self.old_occupied_addresses.clone().for_each(|(uni, adr)| {
                let _ = state.with_address_index_mut(|index| index.remove(uni, adr));
            });
            self.new_occupied_addresses
                .clone()
                .enumerate()
                .for_each(|(offset, (uni, adr))| {
                    let _ = state
                        .with_address_index_mut(|index| index.insert(uni, adr, self.id, offset));
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
                let _ = state.with_address_index_mut(|index| index.remove(uni, adr));
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

mod effect_specs {
    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::effects::{EffectSpec, EffectSpecChange, EffectSpecId};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AddEffectSpecCommand {
        spec: EffectSpec,
    }

    impl AddEffectSpecCommand {
        pub fn new(spec: EffectSpec) -> Self {
            Self { spec }
        }
    }

    impl DocCommand for AddEffectSpecCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let id = self.spec.id();
            state.with_effect_specs_mut(|it| it.insert(id, self.spec));
            (
                Box::new(RemoveEffectSpecCommand::new(id)),
                DocEffect::EffectSpecAdded(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // TODO: あまり綺麗じゃない、EffectSpecChangeとIdだけ持ってればいい
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdateEffectSpecCommand {
        old: EffectSpec,
        new: EffectSpec,
        change: EffectSpecChange,
    }

    impl UpdateEffectSpecCommand {
        pub fn new(old: EffectSpec, new: EffectSpec, change: EffectSpecChange) -> Self {
            Self { old, new, change }
        }
    }

    impl DocCommand for UpdateEffectSpecCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let Self { old, new, change } = *self;
            let id = new.id();
            let rev = UpdateEffectSpecCommand::new(new.clone(), old.clone(), change.clone());
            state.with_effect_specs_mut(|it| {
                *it.get_mut(&id).unwrap() = new;
            });
            (Box::new(rev), DocEffect::EffectSpecUpdated(id))
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RemoveEffectSpecCommand(EffectSpecId);

    impl RemoveEffectSpecCommand {
        pub fn new(id: EffectSpecId) -> Self {
            Self(id)
        }
    }

    impl DocCommand for RemoveEffectSpecCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let removed = state.with_effect_specs_mut(|it| it.remove(&self.0).unwrap());
            let id = removed.id();
            (
                Box::new(AddEffectSpecCommand::new(removed)),
                DocEffect::EffectSpecRemoved(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

mod effect_templates {
    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::effects::{EffectTemplate, EffectTemplateChange, EffectTemplateId};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AddEffectTemplateCommand {
        tmpl: EffectTemplate,
    }

    impl AddEffectTemplateCommand {
        pub fn new(tmpl: EffectTemplate) -> Self {
            Self { tmpl }
        }
    }

    impl DocCommand for AddEffectTemplateCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let id = self.tmpl.id();
            state.with_effect_templates_mut(|it| it.insert(id, self.tmpl));
            (
                Box::new(RemoveEffectTemplateCommand::new(id)),
                DocEffect::EffectTemplateAdded(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdateEffectTemplateCommand {
        old: EffectTemplate,
        new: EffectTemplate,
        change: EffectTemplateChange,
    }

    impl UpdateEffectTemplateCommand {
        pub fn new(old: EffectTemplate, new: EffectTemplate, change: EffectTemplateChange) -> Self {
            Self { old, new, change }
        }
    }

    impl DocCommand for UpdateEffectTemplateCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let Self { old, new, change } = *self;
            let id = new.id();
            let rev = UpdateEffectTemplateCommand::new(new.clone(), old.clone(), change.clone());
            state.with_effect_templates_mut(|it| {
                *it.get_mut(&id).unwrap() = new;
            });
            (Box::new(rev), DocEffect::EffectTemplateUpdated(id))
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RemoveEffectTemplateCommand(EffectTemplateId);

    impl RemoveEffectTemplateCommand {
        pub fn new(id: EffectTemplateId) -> Self {
            Self(id)
        }
    }

    impl DocCommand for RemoveEffectTemplateCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let removed = state.with_effect_templates_mut(|it| it.remove(&self.0).unwrap());
            let id = removed.id();
            (
                Box::new(AddEffectTemplateCommand::new(removed)),
                DocEffect::EffectTemplateRemoved(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

mod effects {
    use crate::doc::state::DocState;
    use crate::doc::{DocCommand, DocEffect};
    use crate::effects::{Effect, EffectChange, EffectId};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AddEffectCommand {
        effect: Effect,
    }

    impl AddEffectCommand {
        pub fn new(effect: Effect) -> Self {
            Self { effect }
        }
    }

    impl DocCommand for AddEffectCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let id = self.effect.id();
            state.with_effects_mut(|it| it.insert(id, self.effect));
            (
                Box::new(RemoveEffectCommand::new(id)),
                DocEffect::EffectAdded(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct UpdateEffectCommand {
        old: Effect,
        new: Effect,
        change: EffectChange,
    }

    impl UpdateEffectCommand {
        pub fn new(old: Effect, new: Effect, change: EffectChange) -> Self {
            Self { old, new, change }
        }
    }

    impl DocCommand for UpdateEffectCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let Self { old, new, change } = *self;
            let id = new.id();
            let rev = UpdateEffectCommand::new(new.clone(), old.clone(), change.clone());
            state.with_effects_mut(|it| {
                *it.get_mut(&id).unwrap() = new;
            });
            (Box::new(rev), DocEffect::EffectUpdated(id))
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RemoveEffectCommand(EffectId);

    impl RemoveEffectCommand {
        pub fn new(id: EffectId) -> Self {
            Self(id)
        }
    }

    impl DocCommand for RemoveEffectCommand {
        fn apply(self: Box<Self>, state: &DocState) -> (Box<dyn DocCommand>, DocEffect) {
            let removed = state.with_effects_mut(|it| it.remove(&self.0).unwrap());
            let id = removed.id();
            (
                Box::new(AddEffectCommand::new(removed)),
                DocEffect::EffectRemoved(id),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

mod universes {
    use crate::{
        doc::{DocCommand, DocEffect},
        prelude::UniverseId,
    };

    #[derive(Debug)]
    pub struct AddUniverseCommand;

    impl DocCommand for AddUniverseCommand {
        fn apply(
            self: Box<Self>,
            state: &crate::doc::DocState,
        ) -> (Box<dyn DocCommand>, crate::doc::DocEffect) {
            let new = state
                .with_universes(|it| it.iter().max().copied())
                .map(|max| UniverseId::new(max.value() + 1))
                .unwrap_or(UniverseId::MIN);
            state.with_universes_mut(|it| it.insert(new));

            //todo!()
            (
                Box::new(RemoveUniverseCommand(new)),
                DocEffect::UniverseAdded(new),
            )
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl AddUniverseCommand {
        pub fn new() -> Self {
            Self
        }
    }

    #[derive(Debug)]
    #[allow(unused)]
    pub struct RemoveUniverseCommand(UniverseId);

    impl DocCommand for RemoveUniverseCommand {
        fn apply(
            self: Box<Self>,
            _state: &crate::doc::DocState,
        ) -> (Box<dyn DocCommand>, DocEffect) {
            todo!()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}
