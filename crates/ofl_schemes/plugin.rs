//! Plugin definitions for the Open Fixture Library.
//!
//! This module defines the structure for import/export plugins that convert
//! fixtures between different formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Types
// ============================================================================

/// HTML string lines that will be joined with newlines.
pub type HtmlStringLines = Vec<String>;

/// A URL string.
pub type UrlString = String;

// ============================================================================
// File Locations
// ============================================================================

/// Platform-specific file locations.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FileLocations {
    /// Main/system location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,

    /// User-specific location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl FileLocations {
    /// Creates new file locations.
    pub fn new(main: Option<String>, user: Option<String>) -> Self {
        Self { main, user }
    }
}

/// File locations grouped by platform with additional options.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFileLocations {
    /// Whether subdirectories are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_directories_allowed: Option<bool>,

    /// Platform-specific locations (Windows, Mac OS, Linux).
    #[serde(flatten)]
    pub platforms: HashMap<String, FileLocations>,
}

// ============================================================================
// Plugin
// ============================================================================

/// A plugin definition for importing/exporting fixtures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    /// JSON Schema reference.
    #[serde(rename = "$schema")]
    pub schema: String,

    /// Plugin name.
    pub name: String,

    /// Previous version names/keys for migration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_versions: Option<HashMap<String, String>>,

    /// Description of the plugin (HTML lines).
    pub description: HtmlStringLines,

    /// Links related to the plugin.
    pub links: HashMap<String, UrlString>,

    /// Instructions for using fixtures with this plugin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixture_usage: Option<HtmlStringLines>,

    /// File locations for this plugin's fixture format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_locations: Option<PlatformFileLocations>,

    /// Additional information (HTML lines).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<HtmlStringLines>,

    /// Request for help with this plugin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_wanted: Option<String>,
}

impl Plugin {
    /// Returns the description as a single string joined by newlines.
    pub fn description_text(&self) -> String {
        self.description.join("\n")
    }

    /// Returns the fixture usage as a single string joined by newlines.
    pub fn fixture_usage_text(&self) -> Option<String> {
        self.fixture_usage.as_ref().map(|lines| lines.join("\n"))
    }

    /// Returns the additional info as a single string joined by newlines.
    pub fn additional_info_text(&self) -> Option<String> {
        self.additional_info.as_ref().map(|lines| lines.join("\n"))
    }

    /// Gets a specific link by key.
    pub fn get_link(&self, key: &str) -> Option<&str> {
        self.links.get(key).map(|s| s.as_str())
    }
}
