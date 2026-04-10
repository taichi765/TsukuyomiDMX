use std::{collections::HashMap, sync::RwLock};

use crate::{
    doc::{UniverseSetting, def_registry::FixtureDefRegistry},
    fixture::{Fixture, FixtureId},
    functions::{AppliedFunctionId, Function, FunctionPrototype, FunctionPrototypeId},
    prelude::{DmxAddress, UniverseId},
};

/// Get fixture id by address.
pub type AddressIndex = HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>;

/// Single source of true.
///
/// Maybe similar to DB in web apps.
/// -- it's just a data structure and validating is [`decider`]'s responsibility as same as application server in web apps.
pub(super) struct DocState {
    fixtures: RwLock<HashMap<FixtureId, Fixture>>,
    fixture_defs: RwLock<Box<dyn FixtureDefRegistry>>,
    functions: RwLock<HashMap<AppliedFunctionId, Function>>,
    function_prototypes: RwLock<HashMap<FunctionPrototypeId, FunctionPrototype>>,

    universe_settings: RwLock<HashMap<UniverseId, UniverseSetting>>,
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

            universe_settings: RwLock::new(HashMap::new()),
            address_index: RwLock::new(AddressIndex::new()),
        }
    }

    pub(super) fn load_defs(&self) -> Result<(), std::io::Error> {
        self.fixture_defs.write().unwrap().load()
    }

    define_rwlock_helper!(fixtures, HashMap<FixtureId, Fixture>);
    define_rwlock_helper!(functions, HashMap<AppliedFunctionId, Function>);
    define_rwlock_helper!(function_prototypes, HashMap<FunctionPrototypeId, FunctionPrototype>);
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
}
