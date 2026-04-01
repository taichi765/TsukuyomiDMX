macro_rules! declare_id_newtype {
    ($(#[$outer:meta])* $name:ident) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }


        impl From<uuid::Uuid> for $name {
            fn from(value: uuid::Uuid) -> Self {
                Self(value)
            }
        }

        impl Into<uuid::Uuid> for $name {
            fn into(self) -> uuid::Uuid {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_hyphenated())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::nil())
            }
        }
    };
}

pub mod doc;
pub mod engine;
pub mod fixture;
pub mod fixture_def;
pub mod functions;
pub mod plugins;
pub mod universe;

pub mod prelude {
    pub use super::{
        doc::{Doc, OutputPluginId},
        fixture::{Fixture, FixtureId, MergeMode},
        fixture_def::{
            Capability, CapabilityInner, ChannelDef, FixtureDef, FixtureDefId, FixtureMode,
        },
        functions::FunctionId,
        universe::{DmxAddress, UniverseId},
    };
}
