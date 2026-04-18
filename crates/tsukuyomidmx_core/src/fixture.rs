use serde::{Deserialize, Serialize};

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

// TODO: builderパターン
// TODO: クロスユニバース
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FixtureDto", into = "FixtureDto")]
pub struct Fixture {
    id: FixtureId,
    name: String,
    tags: Vec<FixtureTag>,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_def_id: FixtureDefId,
    fixture_mode: String,
    x: f32,
    y: f32,
}

// TODO: modeが一つ以上あることを保証
// TODO: Modeのnew type
impl Fixture {
    pub fn new(
        name: impl Into<String>,
        universe_id: UniverseId,
        address: DmxAddress,
        fixture_def_id: FixtureDefId,
        fixture_mode: impl Into<String>,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            id: FixtureId::new(),
            name: name.into(),
            tags: Vec::new(),
            universe_id,
            address,
            fixture_def_id,
            fixture_mode: fixture_mode.into(),
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

    pub fn tags(&self) -> &Vec<FixtureTag> {
        &self.tags
    }

    pub fn universe_id(&self) -> UniverseId {
        self.universe_id
    }

    pub fn address(&self) -> DmxAddress {
        self.address
    }

    pub fn fixture_def(&self) -> &FixtureDefId {
        &self.fixture_def_id
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

impl TryFrom<FixtureDto> for Fixture {
    type Error = String;
    fn try_from(value: FixtureDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            tags: value
                .tags
                .iter()
                .map(|str| {
                    FixtureTag::new(str)
                        .ok_or_else(|| format!("tag contains invalid character: {}", str))
                })
                .collect::<Result<_, _>>()?,
            universe_id: value.universe_id,
            address: value.address,
            fixture_def_id: FixtureDefId::try_from(value.fixture_def_id.as_str())?,
            fixture_mode: value.fixture_mode,
            x: value.x,
            y: value.y,
        })
    }
}

/// Fixtureにつけるタグ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureTag(String);

impl FixtureTag {
    /// Returns `None` if `val` contains invalid character.
    ///
    /// Allowed characters are alphabetic characters, numeric characters and '-' and '_'.
    /// See [`char::is_alphabetic()`] and [`char::is_numeric()`] for more information.
    pub fn new(val: impl Into<String>) -> Option<Self> {
        let val = val.into();
        if val
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
        {
            None
        } else {
            Some(Self(val))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize)]
pub struct FixtureDto {
    id: FixtureId,
    name: String,
    tags: Vec<String>,
    universe_id: UniverseId,
    address: DmxAddress,
    fixture_def_id: String,
    fixture_mode: String,
    x: f32,
    y: f32,
}

impl From<Fixture> for FixtureDto {
    fn from(value: Fixture) -> Self {
        Self {
            id: value.id,
            name: value.name,
            tags: value.tags.iter().map(|tag| tag.0.to_owned()).collect(),
            universe_id: value.universe_id,
            address: value.address,
            fixture_def_id: value.fixture_def_id.to_string(),
            fixture_mode: value.fixture_mode,
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_serialized_and_deserizlized_correctly() {
        let fxt = Fixture::new(
            "Test Fixture(0)",
            UniverseId::new(0),
            DmxAddress::MIN,
            FixtureDefId::new("Test Manufacturer".into(), "Test Model".into()),
            "Mode 1",
            20.,
            10.,
        );

        let json = serde_json::to_string_pretty(&fxt).unwrap();
        println!("{}", json);

        let deserialized: Fixture = serde_json::from_str(&json).unwrap();
        assert_eq!(fxt, deserialized);
    }
}
