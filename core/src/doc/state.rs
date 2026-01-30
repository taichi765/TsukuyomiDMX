use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    doc::{OutputMapError, OutputPluginId, ResolveError, ResolvedAddress, UniverseSetting},
    fixture::{Fixture, FixtureId},
    functions::FunctionData,
    prelude::{DmxAddress, FixtureDef, FixtureDefId, FunctionId, UniverseId},
};

/// Single source of true.
///
/// Maybe similar to DB in web apps.
/// - Is just a data structure. Validating is [`decider`]'s responsibility as same as application server in web apps.
pub struct DocState {
    fixtures: Rc<RefCell<HashMap<FixtureId, Fixture>>>,
    fixture_defs: Rc<RefCell<HashMap<FixtureDefId, FixtureDef>>>,
    functions: Rc<RefCell<HashMap<FunctionId, FunctionData>>>,
    universe_settings: HashMap<UniverseId, UniverseSetting>,

    fixture_by_address_index: HashMap<(UniverseId, DmxAddress), (FixtureId, usize)>,
}

/* ---------- public, readonly ---------- */
impl DocState {
    pub fn new() -> Self {
        Self {
            fixtures: Rc::new(RefCell::new(HashMap::new())),
            fixture_defs: Rc::new(RefCell::new(HashMap::new())),
            functions: Rc::new(RefCell::new(HashMap::new())),
            universe_settings: HashMap::new(),

            fixture_by_address_index: HashMap::new(),
        }
    }

    pub fn as_view(&self) -> DocStateView {
        DocStateView(self)
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
        F: FnOnce(&HashMap<FixtureDefId, FixtureDef>) -> R,
    {
        let defs = self.fixture_defs.borrow();
        f(&defs)
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
        F: FnOnce(&HashMap<FixtureId, Fixture>, &HashMap<FixtureDefId, FixtureDef>) -> R,
    {
        let fixtures = self.fixtures.borrow();
        let defs = self.fixture_defs.borrow();
        f(&fixtures, &defs)
    }

    pub(super) fn with_fixtures_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FixtureId, Fixture>) -> R,
    {
        let mut fixtures = self.fixtures.borrow_mut();
        f(&mut fixtures)
    }

    pub(super) fn with_fixture_defs_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FixtureDefId, FixtureDef>) -> R,
    {
        let mut defs = self.fixture_defs.borrow_mut();
        f(&mut defs)
    }

    pub(super) fn with_functions_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<FunctionId, FunctionData>) -> R,
    {
        let mut functions = self.functions.borrow_mut();
        f(&mut functions)
    }

    pub fn resolve_address(
        &self,
        _fixture_id: FixtureId,
        _channel: &str,
    ) -> Result<(UniverseId, ResolvedAddress), ResolveError> {
        /*let fixture = self
            .fixtures
            .get(&fixture_id)
            .ok_or(ResolveError::FixtureNotFound(FixtureNotFound(fixture_id)))?;

        let fixture_def = self.fixture_defs.get(&fixture.fixture_def()).ok_or(
            ResolveError::FixtureDefNotFound(FixtureDefNotFound {
                fixture_id: fixture.id(),
                fixture_def_id: fixture.fixture_def(),
            }),
        )?;
        let mode =
            fixture_def
                .modes()
                .get(fixture.fixture_mode())
                .ok_or(ResolveError::ModeNotFound(ModeNotFound {
                    fixture_def: fixture.fixture_def(),
                    mode: fixture.fixture_mode().into(),
                }))?;
        let channel_offset =
            mode.get_offset_by_channel(channel)
                .ok_or(ResolveError::ChannelNotFound {
                    fixture_def: fixture.fixture_def(),
                    mode: fixture.fixture_mode().into(),
                    channel: channel.into(),
                })?;

        let merge_mode = fixture_def
            .channel_templates()
            .get(channel)
            .unwrap() // TODO: should return Err
            .merge_mode();
        Ok((
            fixture.universe_id(),
            ResolvedAddress {
                merge_mode,
                address: fixture.address().checked_add(channel_offset).unwrap(), //FIXME: unwrap
            },
        ))*/
        todo!("FixtureStoreかDocに移動するかも")
    }

    pub fn get_fixture_by_address(
        &self,
        universe_id: &UniverseId,
        address: DmxAddress,
    ) -> Option<&(FixtureId, usize)> {
        self.fixture_by_address_index.get(&(*universe_id, address))
    }

    /// Returns max address which is occupied by a fixture.
    ///
    /// If there's no fixture in the universe, `None` is returned.
    /// If universe does not exist in the DocStore, `None` is returned.
    pub fn current_max_address(&self, _universe: UniverseId) -> Option<DmxAddress> {
        /*let max_fixture = fixtures
            .filter(|(_, fxt)| fxt.universe_id() == universe)
            .map(|(_, fxt)| fxt)
            .max_by(|a, b| a.address().cmp(&b.address()));
        let Some(max_fixture) = max_fixture else {
            return None;
        };
        let defs = self.fixture_defs.borrow();
        let fixture_def = defs.get(&max_fixture.fixture_def()).unwrap();
        let adr = max_fixture
            .occupied_addresses(fixture_def)
            .expect("invariant: mode must exist")
            .iter()
            .last()
            .unwrap() // This unwrap() is safe because occupied addresses can't be empty
            .to_owned();
        Some(adr)*/
        todo!("FixtureStoreかDocに移動？")
    }
}

impl DocState {
    /// Returns `Some(old_setting)` or `None`
    fn add_universe(&mut self, id: UniverseId) -> Option<UniverseSetting> {
        self.universe_settings.insert(id, UniverseSetting::new())
    }

    /// Same as [std::collections::HashMap::remove()]
    fn remove_universe(&mut self, id: &UniverseId) -> Option<UniverseSetting> {
        self.universe_settings.remove(id)
    }

    /// Returns `true` when plugin already exists.
    fn add_output(
        &mut self,
        universe_id: UniverseId,
        plugin: OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(universe_id))?;
        let is_inserted = setting.output_plugins.insert(plugin);

        Ok(is_inserted)
    }

    /// Returns `true` when plugin was not in the list.
    fn remove_output(
        &mut self,
        universe_id: &UniverseId,
        plugin: &OutputPluginId,
    ) -> Result<bool, OutputMapError> {
        let setting = self
            .universe_settings
            .get_mut(&universe_id)
            .ok_or(OutputMapError::UniverseNotFound(*universe_id))?;
        let is_removed = setting.output_plugins.remove(&plugin);

        Ok(is_removed)
    }
}

pub struct DocStateView<'a>(&'a DocState);

impl<'a> DocStateView<'a> {
    pub fn with_fixtures<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>) -> R,
    {
        self.0.with_fixtures(f)
    }

    pub fn with_fixture_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureDefId, FixtureDef>) -> R,
    {
        self.0.with_fixture_defs(f)
    }

    pub fn with_functions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FunctionId, FunctionData>) -> R,
    {
        self.0.with_functions(f)
    }

    pub fn with_fixtures_and_defs<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<FixtureId, Fixture>, &HashMap<FixtureDefId, FixtureDef>) -> R,
    {
        self.0.with_fixtures_and_defs(f)
    }
}
