//! Channel definitions for DMX fixtures.
//!
//! This module defines the structure of DMX channels, including their capabilities,
//! fine channel aliases, and other properties.

use serde::{Deserialize, Serialize};

use crate::{
    Capability, DmxValue, DmxValuePercent, NoVariablesString, NonEmptyString,
    VariablePixelKeyString,
};

// ============================================================================
// Enums
// ============================================================================

/// DMX value resolution (bit depth).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DmxValueResolution {
    #[serde(rename = "8bit")]
    Bit8,
    #[serde(rename = "16bit")]
    Bit16,
    #[serde(rename = "24bit")]
    Bit24,
}

/// Channel precedence for HTP (Highest Takes Precedence) or LTP (Latest Takes Precedence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Precedence {
    /// Latest Takes Precedence - last value written wins.
    LTP,
    /// Highest Takes Precedence - highest value wins.
    HTP,
}

// ============================================================================
// Value Types
// ============================================================================

/// A DMX value that can be either absolute or percentage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DmxValueOrPercent {
    Absolute(DmxValue),
    Percent(DmxValuePercent),
}

/// Fine channel alias - can be a regular string or a template variable string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FineChannelAlias {
    Regular(NoVariablesString),
    Template(VariablePixelKeyString),
}

// ============================================================================
// Channel Definition
// ============================================================================

/// A DMX channel definition.
///
/// A channel must have either a single `capability` (for the entire DMX range)
/// or multiple `capabilities` (for different DMX ranges).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    /// Channel name. If not set, the channel key is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,

    /// Aliases for fine channels (16-bit, 24-bit, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fine_channel_aliases: Option<Vec<FineChannelAlias>>,

    /// Resolution of DMX values in capabilities.
    /// Only valid when `fine_channel_aliases` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmx_value_resolution: Option<DmxValueResolution>,

    /// Default DMX value for this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<DmxValueOrPercent>,

    /// Value to use when highlighting this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_value: Option<DmxValueOrPercent>,

    /// Whether this channel's value should remain constant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant: Option<bool>,

    /// Channel precedence (LTP or HTP).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precedence: Option<Precedence>,

    /// Single capability covering the entire DMX range.
    /// Mutually exclusive with `capabilities`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<Capability>,

    /// Multiple capabilities for different DMX ranges.
    /// Mutually exclusive with `capability`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<Capability>>,
}

impl Channel {
    /// Creates a new channel with a single capability.
    pub fn with_capability(capability: Capability) -> Self {
        Self {
            name: None,
            fine_channel_aliases: None,
            dmx_value_resolution: None,
            default_value: None,
            highlight_value: None,
            constant: None,
            precedence: None,
            capability: Some(capability),
            capabilities: None,
        }
    }

    /// Creates a new channel with multiple capabilities.
    pub fn with_capabilities(capabilities: Vec<Capability>) -> Self {
        Self {
            name: None,
            fine_channel_aliases: None,
            dmx_value_resolution: None,
            default_value: None,
            highlight_value: None,
            constant: None,
            precedence: None,
            capability: None,
            capabilities: Some(capabilities),
        }
    }

    /// Returns true if this channel has switching capabilities.
    pub fn has_switching(&self) -> bool {
        if let Some(caps) = &self.capabilities {
            caps.iter().any(|c| match c {
                Capability::NoFunction { common, .. }
                | Capability::ShutterStrobe { common, .. }
                | Capability::StrobeSpeed { common, .. }
                | Capability::StrobeDuration { common, .. }
                | Capability::Intensity { common, .. }
                | Capability::ColorIntensity { common, .. }
                | Capability::ColorPreset { common, .. }
                | Capability::ColorTemperature { common, .. }
                | Capability::Pan { common, .. }
                | Capability::PanContinuous { common, .. }
                | Capability::Tilt { common, .. }
                | Capability::TiltContinuous { common, .. }
                | Capability::PanTiltSpeed { common, .. }
                | Capability::WheelSlot { common, .. }
                | Capability::WheelShake { common, .. }
                | Capability::WheelSlotRotation { common, .. }
                | Capability::WheelRotation { common, .. }
                | Capability::Effect { common, .. }
                | Capability::EffectSpeed { common, .. }
                | Capability::EffectDuration { common, .. }
                | Capability::EffectParameter { common, .. }
                | Capability::SoundSensitivity { common, .. }
                | Capability::BeamAngle { common, .. }
                | Capability::BeamPosition { common, .. }
                | Capability::Focus { common, .. }
                | Capability::Zoom { common, .. }
                | Capability::Iris { common, .. }
                | Capability::IrisEffect { common, .. }
                | Capability::Frost { common, .. }
                | Capability::FrostEffect { common, .. }
                | Capability::Prism { common, .. }
                | Capability::PrismRotation { common, .. }
                | Capability::BladeInsertion { common, .. }
                | Capability::BladeRotation { common, .. }
                | Capability::BladeSystemRotation { common, .. }
                | Capability::Fog { common, .. }
                | Capability::FogOutput { common, .. }
                | Capability::FogType { common, .. }
                | Capability::Rotation { common, .. }
                | Capability::Speed { common, .. }
                | Capability::Time { common, .. }
                | Capability::Maintenance { common, .. }
                | Capability::Generic { common, .. } => common.switch_channels.is_some(),
            })
        } else {
            false
        }
    }
}
