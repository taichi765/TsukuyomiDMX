use std::{cell::RefCell, collections::HashMap};

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
    fixtures: RefCell<HashMap<FixtureId, Fixture>>,
    fixture_defs: RefCell<Box<dyn FixtureDefRegistry>>,
    functions: RefCell<HashMap<FunctionId, FunctionData>>,

    universe_settings: HashMap<UniverseId, UniverseSetting>, // TODO: これもRefCell化するかも
    address_index: RefCell<AddressIndex>,
}

impl DocState {
    pub fn new(def_registry: Box<dyn FixtureDefRegistry>) -> Self {
        Self {
            fixtures: RefCell::new(HashMap::new()),
            fixture_defs: RefCell::new(def_registry),
            functions: RefCell::new(HashMap::new()),

            universe_settings: HashMap::new(),
            address_index: RefCell::new(AddressIndex::new()),
        }
    }

    pub(super) fn load_defs(&self) -> Result<(), std::io::Error> {
        self.fixture_defs.borrow_mut().load()
    }

    pub fn universe_settings(&self) -> &HashMap<UniverseId, UniverseSetting> {
        &self.universe_settings
    }

    pub fn with_fixtures<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>) -> R,
    {
        let fixtures = self.fixtures.borrow();
        f(&fixtures)
    }

    pub fn with_fixture_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn FixtureDefRegistry) -> R,
    {
        let defs = self.fixture_defs.borrow();
        f(&(**defs))
    }

    pub fn with_functions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FunctionId, FunctionData>) -> R,
    {
        let functions = self.functions.borrow();
        f(&functions)
    }

    pub fn with_fixtures_and_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>, &dyn FixtureDefRegistry) -> R,
    {
        let fixtures = self.fixtures.borrow();
        let defs = self.fixture_defs.borrow();
        f(&fixtures, &(**defs))
    }

    pub fn with_address_index<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AddressIndex) -> R,
    {
        let index = self.address_index.borrow();
        f(&index)
    }

    pub(super) fn with_fixtures_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FixtureId, Fixture>) -> R,
    {
        let mut fixtures = self.fixtures.borrow_mut();
        f(&mut fixtures)
    }

    #[allow(unused)]
    pub(super) fn with_functions_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FunctionId, FunctionData>) -> R,
    {
        let mut functions = self.functions.borrow_mut();
        f(&mut functions)
    }

    pub(super) fn with_address_index_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut AddressIndex) -> R,
    {
        let mut index = self.address_index.borrow_mut();
        f(&mut index)
    }
}
