use thiserror::Error;

use crate::{
    doc::def_registry,
    effects::{EffectId, EffectSpecId, EffectTemplateId},
    fixture::FixtureId,
    fixture_def::FixtureDefId,
    universe::{DmxAddress, UniverseId},
};

/// Error type for [super::Doc::resolve_address()]
#[derive(Debug, Error)]
pub enum ResolveError {
    #[error(transparent)]
    FixtureNotFound(#[from] FixtureNotFoundError),
    #[error("cannot find channel {channel} in mode {mode} of {fixture_def:?}")]
    ChannelNotFound {
        fixture_def: FixtureDefId,
        mode: String,
        channel: String,
    },
    #[error("cannot find channel with offset {offset} in mode {mode} of def {fixture_def:?}")]
    ChannelWithOffsetNotFound {
        fixture_def: FixtureDefId,
        mode: String,
        offset: usize,
    },
}

#[derive(Debug, Error)]
#[error("cannot find fixture {0:?}")]
pub struct FixtureNotFoundError(pub FixtureId);

#[derive(Debug, Error)]
#[error("cannot find fixture definition {fixture_def_id:?} for fixture {fixture_id:?}: {source:?}")]
pub struct FixtureDefNotFoundError {
    pub fixture_id: FixtureId,
    pub fixture_def_id: FixtureDefId,
    pub source: def_registry::FixtureDefLookupError,
}

#[derive(Debug, Error)]
#[error("cannot find mode {mode} in the definition {fixture_def:?}")]
pub struct ModeNotFoundError {
    pub fixture_def: FixtureDefId,
    pub mode: String,
}

#[derive(Debug, Error)]
pub enum OutputMapError {
    #[error("there was no universe {0:?}")]
    UniverseNotFound(UniverseId),
}

/// Error type for [super::Doc::insert_fixture()]
#[derive(Debug, Error)]
pub enum FixtureAddError {
    #[error(transparent)]
    FixtureDefNotFound(#[from] FixtureDefNotFoundError),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFoundError),
    #[error(transparent)]
    AddressValidateError(#[from] ValidateError),
    #[error("fixture with id {0:?} already exists")]
    FixtureAlreadyExists(FixtureId),
}

#[derive(Debug, Error)]
pub enum FixtureUpdateError {
    #[error(transparent)]
    FixtureNotFound(#[from] FixtureNotFoundError),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFoundError),
    #[error(transparent)]
    AddressValidateError(#[from] ValidateError),
}

/// Error type for [super::Doc::validate_fixture_address_uniqueness()]
#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("{} address conflicted",.0.len())]
    AddressConflicted(Vec<AddressConflictedError>),
}

/// Internal error type for [`ValidateError`]
#[derive(Debug, Error)]
#[error(
    "address conflicted(Universe{universe:?}:{address:?}): channel {existing_offset} of fixture {existing_fixture_id:?}\
    and channel {new_offset} of fixture {new_fixture_id:?}"
)]
pub struct AddressConflictedError {
    pub universe: UniverseId,
    pub address: DmxAddress,
    pub existing_fixture_id: FixtureId,
    pub existing_offset: usize,
    pub new_fixture_id: FixtureId,
    pub new_offset: usize,
}

#[derive(Debug, Error)]
pub enum FixtureRemoveError {
    #[error(transparent)]
    FixtureNotFound(#[from] FixtureNotFoundError),
}

#[derive(Debug, Error)]
pub enum AddEffectSpecError {
    #[error("effect spec id {0:?} is already used")]
    IdAlreadyUsed(EffectSpecId),
}

#[derive(Debug, Error)]
pub enum UpdateEffectSpecError {
    #[error("cannot find effect spec {0:?}")]
    NotFound(EffectSpecId),
}

#[derive(Debug, Error)]
pub enum RemoveEffectSpecError {
    #[error("cannot find effect spec {0:?}")]
    NotFound(EffectSpecId),
}

#[derive(Debug, Error)]
pub enum AddEffectTemplateError {
    #[error("effect template id {0:?} is already used")]
    IdAlreadyUsed(EffectTemplateId),
}

#[derive(Debug, Error)]
pub enum UpdateEffectTemplateError {
    #[error("cannot find effect template {0:?}")]
    NotFound(EffectTemplateId),
}

#[derive(Debug, Error)]
pub enum RemoveEffectTemplateError {
    #[error("cannot find effect template {0:?}")]
    NotFound(EffectTemplateId),
}

#[derive(Debug, Error)]
pub enum AddEffectError {
    // TODO: idを使っているfunctionの場所とか出したい
    #[error("effect id {0:?} is already used")]
    IdAlreadyUsed(EffectId),
}

#[derive(Debug, Error)]
pub enum UpdateEffectError {
    #[error("cannot find effect {0:?}")]
    NotFound(EffectId),
}

#[derive(Debug, Error)]
pub enum RemoveEffectError {
    #[error("cannot find effect {0:?}")]
    NotFound(EffectId),
}

#[derive(Debug, Error)]
pub enum AddressIndexConstructError {
    #[error(transparent)]
    FixtureDefNotFound(#[from] FixtureDefNotFoundError),
}
