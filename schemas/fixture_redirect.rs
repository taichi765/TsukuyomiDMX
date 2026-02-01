//! Fixture redirect definitions.
//!
//! This module defines the structure for fixture redirects, which point
//! from one fixture to another (e.g., when a fixture is renamed or is
//! the same as another brand's fixture).

use serde::{Deserialize, Serialize};

use crate::NonEmptyString;

/// Reason for the fixture redirect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RedirectReason {
    /// The fixture was renamed (name change only).
    FixtureRenamed,
    /// The fixture is identical to a different brand's fixture.
    SameAsDifferentBrand,
}

/// A fixture redirect definition.
///
/// Redirects are used when a fixture file should point to another fixture,
/// either because the fixture was renamed or because it's identical to
/// a fixture from a different brand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureRedirect {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Fixture name (unique within manufacturer).
    pub name: NonEmptyString,

    /// Target fixture in "manufacturer/fixture" format.
    /// Pattern: `^[a-z0-9\-]+/[a-z0-9\-]+$`
    pub redirect_to: String,

    /// Reason for the redirect.
    pub reason: RedirectReason,
}

impl FixtureRedirect {
    /// Creates a new fixture redirect.
    pub fn new(
        name: impl Into<String>,
        redirect_to: impl Into<String>,
        reason: RedirectReason,
    ) -> Self {
        Self {
            schema: None,
            name: name.into(),
            redirect_to: redirect_to.into(),
            reason,
        }
    }

    /// Creates a redirect for a renamed fixture.
    pub fn renamed(name: impl Into<String>, redirect_to: impl Into<String>) -> Self {
        Self::new(name, redirect_to, RedirectReason::FixtureRenamed)
    }

    /// Creates a redirect for a fixture that's the same as another brand.
    pub fn same_as_different_brand(
        name: impl Into<String>,
        redirect_to: impl Into<String>,
    ) -> Self {
        Self::new(name, redirect_to, RedirectReason::SameAsDifferentBrand)
    }

    /// Returns the manufacturer key from the redirect target.
    pub fn target_manufacturer(&self) -> Option<&str> {
        self.redirect_to.split('/').next()
    }

    /// Returns the fixture key from the redirect target.
    pub fn target_fixture(&self) -> Option<&str> {
        self.redirect_to.split('/').nth(1)
    }

    /// Returns the redirect target as a tuple of (manufacturer, fixture).
    pub fn target(&self) -> Option<(&str, &str)> {
        let mut parts = self.redirect_to.split('/');
        match (parts.next(), parts.next()) {
            (Some(mfr), Some(fix)) => Some((mfr, fix)),
            _ => None,
        }
    }
}
