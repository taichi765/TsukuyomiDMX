use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use crate::{
    doc::{AddressIndexConstructError, FixtureDefNotFoundError, def_registry::FixtureDefRegistry},
    effects::{Effect, EffectId, EffectSpec, EffectSpecId},
    fixture::{Fixture, FixtureId},
    prelude::{DmxAddress, UniverseId},
};

/// Get fixture id by address.
#[derive(derive_more::Deref)]
pub struct AddressIndex(HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>);

/// Single source of true.
///
/// Maybe similar to DB in web apps.
/// -- it's just a data structure and validating is [`decider`]'s responsibility as same as application server in web apps.
pub(super) struct DocState {
    fixtures: RwLock<HashMap<FixtureId, Fixture>>,
    fixture_defs: RwLock<Box<dyn FixtureDefRegistry>>,
    functions: RwLock<HashMap<EffectId, Effect>>,
    function_prototypes: RwLock<HashMap<EffectSpecId, EffectSpec>>,
    universes: RwLock<HashSet<UniverseId>>,

    address_index: RwLock<AddressIndex>,
}

/// with_xxx(|it|it.get()...)のようなメソッドを定義する
macro_rules! define_rwlock_helper {
    ($prop: ident, $typ: ty) => {
        paste::paste! {
            pub fn [<with_ $prop>]<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&$typ) -> R,
            {
                f(&self.$prop.read().unwrap())
            }

            pub(super) fn [<with_ $prop _mut>]<F, R>(&self, f: F) -> R
                where
                    F: FnOnce(&mut $typ) -> R,
            {
                f(&mut self.$prop.write().unwrap())
            }
        }
    };
}

impl DocState {
    pub fn new(def_registry: Box<dyn FixtureDefRegistry>) -> Self {
        Self {
            fixtures: RwLock::new(HashMap::new()),
            fixture_defs: RwLock::new(def_registry),
            functions: RwLock::new(HashMap::new()),
            function_prototypes: RwLock::new(HashMap::new()),
            universes: RwLock::new(HashSet::new()),

            address_index: RwLock::new(AddressIndex::new()),
        }
    }

    pub fn from_existing_data(
        def_registry: Box<dyn FixtureDefRegistry>,
        fixtures: HashMap<FixtureId, Fixture>,
        functions: HashMap<EffectId, Effect>,
        function_prototypes: HashMap<EffectSpecId, EffectSpec>,
        universes: HashSet<UniverseId>,
    ) -> Result<Self, AddressIndexConstructError> {
        let index = AddressIndex::from_fixtures(&fixtures, def_registry.as_ref())?;
        Ok(Self {
            fixtures: RwLock::new(fixtures),
            fixture_defs: RwLock::new(def_registry),
            functions: RwLock::new(functions),
            function_prototypes: RwLock::new(function_prototypes),
            universes: RwLock::new(universes),
            address_index: RwLock::new(index),
        })
    }

    pub(super) fn load_defs(&self) -> Result<(), std::io::Error> {
        self.fixture_defs.write().unwrap().load()
    }

    define_rwlock_helper!(fixtures, HashMap<FixtureId, Fixture>);
    define_rwlock_helper!(functions, HashMap<EffectId, Effect>);
    define_rwlock_helper!(function_prototypes, HashMap<EffectSpecId, EffectSpec>);
    define_rwlock_helper!(address_index, AddressIndex);
    define_rwlock_helper!(universes, HashSet<UniverseId>);

    pub fn with_fixture_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn FixtureDefRegistry) -> R,
    {
        f(&(**self.fixture_defs.read().unwrap()))
    }

    pub fn with_fixtures_and_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>, &dyn FixtureDefRegistry) -> R,
    {
        let fixtures = self.fixtures.read().unwrap();
        let defs = self.fixture_defs.read().unwrap();
        f(&fixtures, &(**defs))
    }

    pub fn universes(&self) -> Vec<UniverseId> {
        self.universes.read().unwrap().iter().cloned().collect()
    }
}

impl AddressIndex {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_fixtures(
        fixtures: &HashMap<FixtureId, Fixture>,
        defs: &dyn FixtureDefRegistry,
    ) -> Result<Self, AddressIndexConstructError> {
        let data = fixtures
            .iter()
            .try_fold(HashMap::new(), |mut acc, (_, fxt)| {
                let def = defs
                    .get(fxt.fixture_def())
                    .map_err(|e| FixtureDefNotFoundError {
                        fixture_id: fxt.id(),
                        fixture_def_id: fxt.fixture_def().to_owned(),
                        source: e,
                    })?;
                def.mode(fxt.fixture_mode())
                    .unwrap()
                    .occupied_addresses(fxt.universe_id(), fxt.address())
                    .enumerate()
                    .for_each(|(offset, (univ, adr))| {
                        acc.insert((univ, adr), (fxt.id(), offset));
                    });
                Ok::<_, AddressIndexConstructError>(acc)
            })?;
        Ok(Self(data))
    }

    pub fn insert(
        &mut self,
        univ: UniverseId,
        adr: DmxAddress,
        fxt_id: FixtureId,
        offset: usize,
    ) -> Option<(FixtureId, usize)> {
        self.0.insert((univ, adr), (fxt_id, offset))
    }

    pub fn remove(&mut self, univ: UniverseId, adr: DmxAddress) -> Option<(FixtureId, usize)> {
        self.0.remove(&(univ, adr))
    }

    pub fn get(&self, univ: UniverseId, adr: DmxAddress) -> Option<&(FixtureId, usize)> {
        self.0.get(&(univ, adr))
    }

    pub fn iter(&self) -> impl Iterator<Item = (UniverseId, DmxAddress, FixtureId, usize)> {
        self.0
            .iter()
            .map(|((univ, adr), (fxt_id, offset))| (*univ, *adr, *fxt_id, *offset))
    }
}
