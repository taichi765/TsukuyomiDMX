use crate::{
    fixture_def::FixtureDefId,
    universe::{DmxAddress, UniverseId},
};

declare_id_newtype!(FixtureId);

#[derive(Clone, Copy, Debug)]
pub enum MergeMode {
    HTP,
    LTP,
}

// TODO: クロスユニバース
#[derive(Debug, Clone)]
pub struct Fixture {
    id: FixtureId,
    name: String,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_def_id: FixtureDefId,
    fixture_mode: String,
    x: f32,
    y: f32,
}

// TODO: modeが一つ以上あることを保証
impl Fixture {
    pub fn new(
        name: impl Into<String>,
        universe_id: UniverseId,
        address: DmxAddress,
        fixture_def_id: FixtureDefId,
        fixture_mode: String,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            id: FixtureId::new(),
            name: name.into(),
            universe_id,
            address,
            fixture_def_id,
            fixture_mode,
            x,
            y,
        }
    }

    pub fn id(&self) -> FixtureId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn universe_id(&self) -> UniverseId {
        self.universe_id
    }

    pub fn address(&self) -> DmxAddress {
        self.address
    }

    pub fn fixture_def(&self) -> FixtureDefId {
        self.fixture_def_id
    }

    pub fn fixture_mode(&self) -> &str {
        &self.fixture_mode
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub(crate) fn apply_change(&mut self, change: FixtureChange) {
        match change {
            FixtureChange::Rename(name) => self.name = name,
            FixtureChange::Universe(uni) => self.universe_id = uni,
            FixtureChange::Address(adr) => self.address = adr,
            FixtureChange::Mode(mode) => self.fixture_mode = mode,
            FixtureChange::Position(x, y) => {
                self.x = x;
                self.y = y;
            }
        }
    }
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: "fixture".to_string(),
            universe_id: Default::default(),
            address: Default::default(),
            fixture_def_id: Default::default(),
            fixture_mode: "mode".to_string(),
            x: 0.,
            y: 0.,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FixtureChange {
    Rename(String),
    Universe(UniverseId),
    Address(DmxAddress),
    // TODO: 必要か？　FixtureDef(FixtureDefId),
    Mode(String),
    Position(f32, f32),
}

impl FixtureChange {
    /// 逆操作を生成する
    pub(crate) fn inverse_from(&self, fixture: &Fixture) -> FixtureChange {
        match self {
            FixtureChange::Rename(_) => FixtureChange::Rename(fixture.name().to_string()),
            FixtureChange::Universe(_) => FixtureChange::Universe(fixture.universe_id()),
            FixtureChange::Address(_) => FixtureChange::Address(fixture.address()),
            FixtureChange::Mode(_) => FixtureChange::Mode(fixture.fixture_mode().to_string()),
            FixtureChange::Position(_, _) => {
                let (x, y) = fixture.pos();
                FixtureChange::Position(x, y)
            }
        }
    }
}
