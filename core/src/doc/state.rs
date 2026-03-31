use std::{collections::HashMap, sync::RwLock};

use crate::{
    doc::{UniverseSetting, def_registry::FixtureDefRegistry},
    fixture::{Fixture, FixtureId},
    functions::FunctionData,
    prelude::{DmxAddress, FunctionId, UniverseId},
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
    functions: RwLock<HashMap<FunctionId, FunctionData>>,

    universe_settings: RwLock<HashMap<UniverseId, UniverseSetting>>,
    address_index: RwLock<AddressIndex>,
}

impl DocState {
    pub fn new(def_registry: Box<dyn FixtureDefRegistry>) -> Self {
        Self {
            fixtures: RwLock::new(HashMap::new()),
            fixture_defs: RwLock::new(def_registry),
            functions: RwLock::new(HashMap::new()),

            universe_settings: RwLock::new(HashMap::new()),
            address_index: RwLock::new(AddressIndex::new()),
        }
    }

    pub(super) fn load_defs(&self) -> Result<(), std::io::Error> {
        self.fixture_defs.write().unwrap().load()
    }

    /*pub fn universe_settings(&self) -> &HashMap<UniverseId, UniverseSetting> {
        &self.universe_settings
    }*/

    pub fn with_fixtures<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>) -> R,
    {
        let fixtures = self.fixtures.read().unwrap();
        f(&fixtures)
    }

    pub fn with_fixture_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn FixtureDefRegistry) -> R,
    {
        let defs = self.fixture_defs.read().unwrap();
        f(&(**defs))
    }

    pub fn with_functions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FunctionId, FunctionData>) -> R,
    {
        let functions = self.functions.read().unwrap();
        f(&functions)
    }

    pub fn with_fixtures_and_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>, &dyn FixtureDefRegistry) -> R,
    {
        let fixtures = self.fixtures.read().unwrap();
        let defs = self.fixture_defs.read().unwrap();
        f(&fixtures, &(**defs))
    }

    pub fn with_address_index<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AddressIndex) -> R,
    {
        let index = self.address_index.read().unwrap();
        f(&index)
    }

    pub(super) fn with_fixtures_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FixtureId, Fixture>) -> R,
    {
        let mut fixtures = self.fixtures.write().unwrap();
        f(&mut fixtures)
    }

    #[allow(unused)]
    pub(super) fn with_functions_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FunctionId, FunctionData>) -> R,
    {
        let mut functions = self.functions.write().unwrap();
        f(&mut functions)
    }

    pub(super) fn with_address_index_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut AddressIndex) -> R,
    {
        let mut index = self.address_index.write().unwrap();
        f(&mut index)
    }
}
