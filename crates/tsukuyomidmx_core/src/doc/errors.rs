use thiserror::Error;

use crate::{
    doc::def_registry,
    fixture::FixtureId,
    fixture_def::FixtureDefId,
    prelude::AppliedFunctionId,
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
pub enum AddFunctionError {
    // TODO: idを使っているfunctionの場所とか出したい
    #[error("function id {0:?} is already used")]
    IdAlreadyUsed(AppliedFunctionId),
}

#[derive(Debug, Error)]
pub enum RemoveFunctionError {
    #[error("cannot find function {0:?}")]
    FunctionNotFound(AppliedFunctionId),
}

#[derive(Debug, Error)]
pub enum AddOutputPluginError {
    #[error("there were no universe with id {0:?}")]
    UniverseNotFound(UniverseId),
}

#[derive(Debug, Error)]
pub enum RemoveOutputPluginError {}
