//! Wheel slot definitions.
//!
//! This module defines the various types of slots that can appear on
//! color wheels, gobo wheels, and other rotating wheels in fixtures.

use serde::{Deserialize, Serialize};

use crate::{
    ColorString, ColorTemperature, GoboResourceString, IrisPercent, NonEmptyString, Percent,
};

/// A slot on a wheel (color wheel, gobo wheel, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WheelSlot {
    /// Open position (no filter/gobo).
    Open,

    /// Closed position (blocks light).
    Closed,

    /// A color filter slot.
    Color {
        /// Optional name for the color.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,

        /// Color values (hex strings).
        #[serde(skip_serializing_if = "Option::is_none")]
        colors: Option<Vec<ColorString>>,

        /// Color temperature of the filter.
        #[serde(rename = "colorTemperature", skip_serializing_if = "Option::is_none")]
        color_temperature: Option<ColorTemperature>,
    },

    /// A gobo (pattern) slot.
    Gobo {
        /// Resource path to the gobo image.
        #[serde(skip_serializing_if = "Option::is_none")]
        resource: Option<GoboResourceString>,

        /// Name of the gobo.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,
    },

    /// A prism slot.
    Prism {
        /// Name of the prism.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,

        /// Number of facets on the prism.
        #[serde(skip_serializing_if = "Option::is_none")]
        facets: Option<u32>,
    },

    /// An iris slot.
    Iris {
        /// How open the iris is.
        #[serde(rename = "openPercent", skip_serializing_if = "Option::is_none")]
        open_percent: Option<IrisPercent>,
    },

    /// A frost filter slot.
    Frost {
        /// Intensity of the frost effect.
        #[serde(rename = "frostIntensity", skip_serializing_if = "Option::is_none")]
        frost_intensity: Option<Percent>,
    },

    /// Start of an animation gobo (split gobo).
    AnimationGoboStart {
        /// Name of the animation gobo.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,
    },

    /// End of an animation gobo (split gobo).
    AnimationGoboEnd,
}

impl WheelSlot {
    /// Creates an open slot.
    pub fn open() -> Self {
        Self::Open
    }

    /// Creates a closed slot.
    pub fn closed() -> Self {
        Self::Closed
    }

    /// Creates a color slot with a single color.
    pub fn color(color: impl Into<String>) -> Self {
        Self::Color {
            name: None,
            colors: Some(vec![color.into()]),
            color_temperature: None,
        }
    }

    /// Creates a named color slot.
    pub fn named_color(name: impl Into<String>, colors: Vec<String>) -> Self {
        Self::Color {
            name: Some(name.into()),
            colors: Some(colors),
            color_temperature: None,
        }
    }

    /// Creates a gobo slot with a resource path.
    pub fn gobo(resource: impl Into<String>) -> Self {
        Self::Gobo {
            resource: Some(resource.into()),
            name: None,
        }
    }

    /// Creates a named gobo slot.
    pub fn named_gobo(name: impl Into<String>) -> Self {
        Self::Gobo {
            resource: None,
            name: Some(name.into()),
        }
    }

    /// Creates a prism slot.
    pub fn prism(facets: u32) -> Self {
        Self::Prism {
            name: None,
            facets: Some(facets),
        }
    }

    /// Returns true if this is an open slot.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Returns true if this is a closed slot.
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Returns true if this is a color slot.
    pub fn is_color(&self) -> bool {
        matches!(self, Self::Color { .. })
    }

    /// Returns true if this is a gobo slot.
    pub fn is_gobo(&self) -> bool {
        matches!(self, Self::Gobo { .. })
    }

    /// Returns the name of the slot, if any.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Color { name, .. } => name.as_deref(),
            Self::Gobo { name, .. } => name.as_deref(),
            Self::Prism { name, .. } => name.as_deref(),
            Self::AnimationGoboStart { name } => name.as_deref(),
            _ => None,
        }
    }
}
