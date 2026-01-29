use thiserror::Error;

use crate::{
    fixture::FixtureId,
    fixture_def::FixtureDefId,
    universe::{DmxAddress, UniverseId},
};

/// Error type for [super::Doc::resolve_address()]
#[derive(Debug, Error)]
pub enum ResolveError {
    #[error(transparent)]
    FixtureNotFound(#[from] FixtureNotFound),
    #[error(transparent)]
    FixtureDefNotFound(#[from] FixtureDefNotFound),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFound),
    #[error("cannot find channel {channel} in mode {mode} of {fixture_def:?}")]
    ChannelNotFound {
        fixture_def: FixtureDefId,
        mode: String,
        channel: String,
    },
}

#[derive(Debug, Error)]
#[error("cannot find fixture {0:?}")]
pub struct FixtureNotFound(pub FixtureId);

#[derive(Debug, Error)]
#[error("cannot find fixture definition {fixture_def_id:?} for fixture {fixture_id:?}")]
pub struct FixtureDefNotFound {
    pub fixture_id: FixtureId,
    pub fixture_def_id: FixtureDefId,
}

#[derive(Debug, Error)]
#[error("cannot find mode {mode} in the definition {fixture_def:?}")]
pub struct ModeNotFound {
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
    FixtureDefNotFound(#[from] FixtureDefNotFound),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFound),
    #[error(transparent)]
    AddressValidateError(#[from] ValidateError),
    #[error("fixture with id {0:?} already exists")]
    FixtureAlreadyExists(FixtureId),
}

#[derive(Debug, Error)]
pub enum FixtureUpdateError {
    #[error(transparent)]
    FixtureNotFound(#[from] FixtureNotFound),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFound),
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
    "address conflicted: channel {existing_offset} of fixture {existing_fixture_id:?}\
    and channel {new_offset} of fixture {new_fixture_id:?}"
)]
pub struct AddressConflictedError {
    pub address: DmxAddress,
    pub existing_fixture_id: FixtureId,
    pub existing_offset: usize,
    pub new_fixture_id: FixtureId,
    pub new_offset: usize,
}

#[derive(Debug, Error)]
pub enum FixtureRemoveError {
    #[error(transparent)]
    FixtureDefNotFound(#[from] FixtureDefNotFound),
    #[error(transparent)]
    ModeNotFound(#[from] ModeNotFound),
}
