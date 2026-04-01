//! Matrix definitions for pixel-based fixtures.
//!
//! This module defines the structure for LED matrices/pixel arrays,
//! including pixel counts, keys, and grouping constraints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::NoVariablesString;

// ============================================================================
// Pixel Number Constraints
// ============================================================================

/// Constraint for filtering pixels by position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PixelNumberConstraint {
    /// Exact position: `=N` (e.g., "=1", "=5")
    Exact(ExactPosition),
    /// Minimum position: `>=N` (e.g., ">=1", ">=5")
    Minimum(MinimumPosition),
    /// Maximum position: `<=N` (e.g., "<=10", "<=5")
    Maximum(MaximumPosition),
    /// Divisible by N: `Nn` (e.g., "2n", "3n")
    Divisible(DivisiblePosition),
    /// Divisible with remainder: `Nn+M` (e.g., "2n+1", "3n+2")
    DivisibleWithRemainder(DivisibleWithRemainderPosition),
    /// Even positions
    Even(EvenPosition),
    /// Odd positions
    Odd(OddPosition),
}

/// Exact position constraint: `=N`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExactPosition(pub u32);

impl Serialize for ExactPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("={}", self.0))
    }
}

impl<'de> Deserialize<'de> for ExactPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(num) = s.strip_prefix('=') {
            num.parse()
                .map(ExactPosition)
                .map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom("expected '=N' format"))
        }
    }
}

/// Minimum position constraint: `>=N`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MinimumPosition(pub u32);

impl Serialize for MinimumPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(">={}", self.0))
    }
}

impl<'de> Deserialize<'de> for MinimumPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(num) = s.strip_prefix(">=") {
            num.parse()
                .map(MinimumPosition)
                .map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom("expected '>=N' format"))
        }
    }
}

/// Maximum position constraint: `<=N`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaximumPosition(pub u32);

impl Serialize for MaximumPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("<={}", self.0))
    }
}

impl<'de> Deserialize<'de> for MaximumPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(num) = s.strip_prefix("<=") {
            num.parse()
                .map(MaximumPosition)
                .map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom("expected '<=N' format"))
        }
    }
}

/// Divisible position constraint: `Nn` (every Nth position)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DivisiblePosition(pub u32);

impl Serialize for DivisiblePosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}n", self.0))
    }
}

impl<'de> Deserialize<'de> for DivisiblePosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(num) = s.strip_suffix('n') {
            if !num.contains('+') {
                return num
                    .parse()
                    .map(DivisiblePosition)
                    .map_err(serde::de::Error::custom);
            }
        }
        Err(serde::de::Error::custom("expected 'Nn' format"))
    }
}

/// Divisible with remainder constraint: `Nn+M`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DivisibleWithRemainderPosition {
    pub divisor: u32,
    pub remainder: u32,
}

impl Serialize for DivisibleWithRemainderPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}n+{}", self.divisor, self.remainder))
    }
}

impl<'de> Deserialize<'de> for DivisibleWithRemainderPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some((divisor_str, remainder_str)) = s.split_once("n+") {
            let divisor = divisor_str.parse().map_err(serde::de::Error::custom)?;
            let remainder = remainder_str.parse().map_err(serde::de::Error::custom)?;
            Ok(DivisibleWithRemainderPosition { divisor, remainder })
        } else {
            Err(serde::de::Error::custom("expected 'Nn+M' format"))
        }
    }
}

/// Even position marker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvenPosition;

impl Serialize for EvenPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("even")
    }
}

impl<'de> Deserialize<'de> for EvenPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "even" {
            Ok(EvenPosition)
        } else {
            Err(serde::de::Error::custom("expected 'even'"))
        }
    }
}

/// Odd position marker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OddPosition;

impl Serialize for OddPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("odd")
    }
}

impl<'de> Deserialize<'de> for OddPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "odd" {
            Ok(OddPosition)
        } else {
            Err(serde::de::Error::custom("expected 'odd'"))
        }
    }
}

// ============================================================================
// Pixel Groups
// ============================================================================

/// Pixel group definition - specifies which pixels belong to a group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PixelGroup {
    /// Explicit list of pixel keys
    Keys(Vec<NoVariablesString>),
    /// All pixels in the matrix
    All(AllPixels),
    /// Constraint-based selection
    Constraints(PixelConstraints),
}

/// Marker for "all pixels"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllPixels;

impl Serialize for AllPixels {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("all")
    }
}

impl<'de> Deserialize<'de> for AllPixels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "all" {
            Ok(AllPixels)
        } else {
            Err(serde::de::Error::custom("expected 'all'"))
        }
    }
}

/// Constraint-based pixel selection.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PixelConstraints {
    /// X-axis constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<Vec<PixelNumberConstraint>>,

    /// Y-axis constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<Vec<PixelNumberConstraint>>,

    /// Z-axis constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<Vec<PixelNumberConstraint>>,

    /// Name pattern constraints (regex patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<String>>,
}

// ============================================================================
// Matrix Definition
// ============================================================================

/// A 3D pixel key - can be a pixel key string or null (for empty positions).
pub type PixelKey = Option<NoVariablesString>;

/// Matrix definition for pixel-based fixtures.
///
/// Must have either `pixel_count` or `pixel_keys`, but not both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix {
    /// Number of pixels in X, Y, Z directions.
    /// Use this for simple rectangular matrices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_count: Option<[u32; 3]>,

    /// Explicit pixel key layout as a 3D array [Z][Y][X].
    /// Use this for non-rectangular or sparse matrices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_keys: Option<Vec<Vec<Vec<PixelKey>>>>,

    /// Named groups of pixels for easier channel mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_groups: Option<HashMap<NoVariablesString, PixelGroup>>,
}

impl Matrix {
    /// Creates a new matrix with the given pixel count.
    pub fn with_count(x: u32, y: u32, z: u32) -> Self {
        Self {
            pixel_count: Some([x, y, z]),
            pixel_keys: None,
            pixel_groups: None,
        }
    }

    /// Creates a new matrix with explicit pixel keys.
    pub fn with_keys(keys: Vec<Vec<Vec<PixelKey>>>) -> Self {
        Self {
            pixel_count: None,
            pixel_keys: Some(keys),
            pixel_groups: None,
        }
    }

    /// Returns the dimensions of the matrix as (X, Y, Z).
    pub fn dimensions(&self) -> Option<(u32, u32, u32)> {
        if let Some([x, y, z]) = self.pixel_count {
            Some((x, y, z))
        } else if let Some(keys) = &self.pixel_keys {
            let z = keys.len() as u32;
            let y = keys.first().map(|row| row.len() as u32).unwrap_or(0);
            let x = keys
                .first()
                .and_then(|row| row.first())
                .map(|col| col.len() as u32)
                .unwrap_or(0);
            Some((x, y, z))
        } else {
            None
        }
    }

    /// Returns the total number of pixels (including null positions).
    pub fn total_positions(&self) -> u32 {
        if let Some((x, y, z)) = self.dimensions() {
            x * y * z
        } else {
            0
        }
    }
}
