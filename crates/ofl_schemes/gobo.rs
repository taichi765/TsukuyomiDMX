//! Gobo definitions.
//!
//! This module defines the structure for gobo (pattern) images in the
//! Open Fixture Library.

use serde::{Deserialize, Serialize};

use crate::{NonEmptyString, UrlString};

/// A gobo (pattern) definition.
///
/// Gobos are patterns or images projected by fixtures, typically through
/// a rotating wheel mechanism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gobo {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Gobo name.
    pub name: NonEmptyString,

    /// Space-separated list of lowercase keywords describing the gobo.
    /// Pattern: `^[a-z]+( [a-z]+)*$`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,

    /// Source of the gobo (URL or description text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<GoboSource>,
}

/// Source information for a gobo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GoboSource {
    /// A URL pointing to the source.
    Url(UrlString),
    /// A text description of the source.
    Text(NonEmptyString),
}

impl Gobo {
    /// Creates a new gobo with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: None,
            name: name.into(),
            keywords: None,
            source: None,
        }
    }

    /// Creates a new gobo with name and keywords.
    pub fn with_keywords(name: impl Into<String>, keywords: impl Into<String>) -> Self {
        Self {
            schema: None,
            name: name.into(),
            keywords: Some(keywords.into()),
            source: None,
        }
    }

    /// Returns the keywords as a vector of individual words.
    pub fn keyword_list(&self) -> Vec<&str> {
        self.keywords
            .as_deref()
            .map(|k| k.split(' ').collect())
            .unwrap_or_default()
    }

    /// Checks if the gobo has a specific keyword.
    pub fn has_keyword(&self, keyword: &str) -> bool {
        self.keyword_list().contains(&keyword)
    }
}
