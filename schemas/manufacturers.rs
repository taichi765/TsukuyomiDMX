//! Manufacturer definitions.
//!
//! This module defines the structure for manufacturer information in the
//! Open Fixture Library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::{NonEmptyMultilineString, NonEmptyString, UrlString};

// ============================================================================
// Manufacturer
// ============================================================================

/// A manufacturer definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manufacturer {
    /// Manufacturer name.
    pub name: NonEmptyString,

    /// Additional comments about the manufacturer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<NonEmptyMultilineString>,

    /// Manufacturer website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<UrlString>,

    /// RDM manufacturer ID (1-32767).
    #[serde(rename = "rdmId", skip_serializing_if = "Option::is_none")]
    pub rdm_id: Option<u16>,
}

impl Manufacturer {
    /// Creates a new manufacturer with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            comment: None,
            website: None,
            rdm_id: None,
        }
    }

    /// Creates a new manufacturer with name and website.
    pub fn with_website(name: impl Into<String>, website: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            comment: None,
            website: Some(website.into()),
            rdm_id: None,
        }
    }
}

// ============================================================================
// Manufacturers Collection
// ============================================================================

/// Collection of manufacturers keyed by manufacturer key.
///
/// The manufacturer key is a lowercase string with hyphens (e.g., "martin", "chauvet-dj").
/// Keys starting with `$` are reserved for special purposes (like `$schema`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Manufacturers {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Manufacturers keyed by their key.
    #[serde(flatten)]
    pub manufacturers: HashMap<String, Manufacturer>,
}

impl Manufacturers {
    /// Creates a new empty manufacturers collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a manufacturer by key.
    pub fn get(&self, key: &str) -> Option<&Manufacturer> {
        self.manufacturers.get(key)
    }

    /// Gets a mutable reference to a manufacturer by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Manufacturer> {
        self.manufacturers.get_mut(key)
    }

    /// Inserts a manufacturer.
    pub fn insert(&mut self, key: impl Into<String>, manufacturer: Manufacturer) {
        self.manufacturers.insert(key.into(), manufacturer);
    }

    /// Returns the number of manufacturers.
    pub fn len(&self) -> usize {
        self.manufacturers.len()
    }

    /// Returns true if there are no manufacturers.
    pub fn is_empty(&self) -> bool {
        self.manufacturers.is_empty()
    }

    /// Iterates over all manufacturers.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Manufacturer)> {
        self.manufacturers.iter()
    }

    /// Iterates over all manufacturer keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.manufacturers.keys()
    }

    /// Iterates over all manufacturer values.
    pub fn values(&self) -> impl Iterator<Item = &Manufacturer> {
        self.manufacturers.values()
    }

    /// Finds a manufacturer by name (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Option<(&String, &Manufacturer)> {
        let name_lower = name.to_lowercase();
        self.manufacturers
            .iter()
            .find(|(_, m)| m.name.to_lowercase() == name_lower)
    }

    /// Finds a manufacturer by RDM ID.
    pub fn find_by_rdm_id(&self, rdm_id: u16) -> Option<(&String, &Manufacturer)> {
        self.manufacturers
            .iter()
            .find(|(_, m)| m.rdm_id == Some(rdm_id))
    }
}

impl Deref for Manufacturers {
    type Target = HashMap<String, Manufacturer>;

    fn deref(&self) -> &Self::Target {
        &self.manufacturers
    }
}

impl DerefMut for Manufacturers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.manufacturers
    }
}

impl IntoIterator for Manufacturers {
    type Item = (String, Manufacturer);
    type IntoIter = std::collections::hash_map::IntoIter<String, Manufacturer>;

    fn into_iter(self) -> Self::IntoIter {
        self.manufacturers.into_iter()
    }
}

impl<'a> IntoIterator for &'a Manufacturers {
    type Item = (&'a String, &'a Manufacturer);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Manufacturer>;

    fn into_iter(self) -> Self::IntoIter {
        self.manufacturers.iter()
    }
}

impl FromIterator<(String, Manufacturer)> for Manufacturers {
    fn from_iter<I: IntoIterator<Item = (String, Manufacturer)>>(iter: I) -> Self {
        Self {
            schema: None,
            manufacturers: iter.into_iter().collect(),
        }
    }
}
