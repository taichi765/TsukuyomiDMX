//! Fixture definitions - the main schema for lighting fixtures.
//!
//! This module defines the complete structure for describing a lighting fixture,
//! including its physical properties, channels, modes, and more.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    Channel, DimensionsXYZ, IsoDateString, Matrix, NoVariablesString, NonEmptyMultilineString,
    NonEmptyString, UrlString, VariablePixelKeyString, WheelSlot,
};

// ============================================================================
// Enums
// ============================================================================

/// Fixture category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "Barrel Scanner")]
    BarrelScanner,
    Blinder,
    #[serde(rename = "Color Changer")]
    ColorChanger,
    Dimmer,
    Effect,
    Fan,
    Flower,
    Hazer,
    Laser,
    Matrix,
    #[serde(rename = "Moving Head")]
    MovingHead,
    #[serde(rename = "Pixel Bar")]
    PixelBar,
    Scanner,
    Smoke,
    Stand,
    Strobe,
    Other,
}

/// DMX connector type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DmxConnector {
    #[serde(rename = "3-pin")]
    Pin3,
    #[serde(rename = "3-pin (swapped +/-)")]
    Pin3Swapped,
    #[serde(rename = "3-pin XLR IP65")]
    Pin3Ip65,
    #[serde(rename = "5-pin")]
    Pin5,
    #[serde(rename = "5-pin XLR IP65")]
    Pin5Ip65,
    #[serde(rename = "3-pin and 5-pin")]
    Pin3And5,
    #[serde(rename = "3.5mm stereo jack")]
    StereoJack,
    #[serde(rename = "RJ45")]
    Rj45,
}

/// Wheel rotation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WheelDirection {
    CW,
    CCW,
}

/// Power connector input/output type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerConnectorMode {
    #[serde(rename = "input only")]
    InputOnly,
    #[serde(rename = "output only")]
    OutputOnly,
    #[serde(rename = "input and output")]
    InputAndOutput,
}

/// Repeat ordering for matrix channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepeatFor {
    Order(RepeatOrder),
    Explicit(Vec<NoVariablesString>),
}

/// Predefined repeat orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepeatOrder {
    #[serde(rename = "eachPixelABC")]
    EachPixelAbc,
    #[serde(rename = "eachPixelXYZ")]
    EachPixelXyz,
    #[serde(rename = "eachPixelXZY")]
    EachPixelXzy,
    #[serde(rename = "eachPixelYXZ")]
    EachPixelYxz,
    #[serde(rename = "eachPixelYZX")]
    EachPixelYzx,
    #[serde(rename = "eachPixelZXY")]
    EachPixelZxy,
    #[serde(rename = "eachPixelZYX")]
    EachPixelZyx,
    #[serde(rename = "eachPixelGroup")]
    EachPixelGroup,
}

/// Channel ordering within matrix channel insert blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChannelOrder {
    PerPixel,
    PerChannel,
}

// ============================================================================
// Physical Properties
// ============================================================================

/// Power connectors specification.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PowerConnectors {
    #[serde(rename = "IEC C13", skip_serializing_if = "Option::is_none")]
    pub iec_c13: Option<PowerConnectorMode>,

    #[serde(rename = "IEC C19", skip_serializing_if = "Option::is_none")]
    pub iec_c19: Option<PowerConnectorMode>,

    #[serde(rename = "powerCON", skip_serializing_if = "Option::is_none")]
    pub powercon: Option<PowerConnectorMode>,

    #[serde(rename = "powerCON TRUE1", skip_serializing_if = "Option::is_none")]
    pub powercon_true1: Option<PowerConnectorMode>,

    #[serde(rename = "powerCON TRUE1 TOP", skip_serializing_if = "Option::is_none")]
    pub powercon_true1_top: Option<PowerConnectorMode>,

    #[serde(rename = "powerCON 32 A", skip_serializing_if = "Option::is_none")]
    pub powercon_32a: Option<PowerConnectorMode>,

    #[serde(rename = "Hardwired", skip_serializing_if = "Option::is_none")]
    pub hardwired: Option<PowerConnectorMode>,

    #[serde(rename = "Proprietary", skip_serializing_if = "Option::is_none")]
    pub proprietary: Option<PowerConnectorMode>,
}

/// Bulb/lamp specifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bulb {
    /// Bulb type (e.g., "LED", "Halogen").
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub bulb_type: Option<NonEmptyString>,

    /// Color temperature in Kelvin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_temperature: Option<f64>,

    /// Luminous flux in lumens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lumens: Option<f64>,
}

/// Lens specifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lens {
    /// Lens name (e.g., "PC", "Fresnel").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,

    /// Beam angle range [min, max] in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degrees_min_max: Option<[f64; 2]>,
}

/// Matrix pixel physical properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixPixels {
    /// Pixel dimensions in mm [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<DimensionsXYZ>,

    /// Spacing between pixels in mm [X, Y, Z].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<DimensionsXYZ>,
}

/// Physical properties of the fixture.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Physical {
    /// Physical dimensions in mm [width, height, depth].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<DimensionsXYZ>,

    /// Weight in kg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,

    /// Power consumption in W.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<f64>,

    /// Power connectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_connectors: Option<PowerConnectors>,

    /// DMX connector type.
    #[serde(rename = "DMXconnector", skip_serializing_if = "Option::is_none")]
    pub dmx_connector: Option<DmxConnector>,

    /// Bulb/lamp specifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulb: Option<Bulb>,

    /// Lens specifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<Lens>,

    /// Matrix pixel properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix_pixels: Option<MatrixPixels>,
}

// ============================================================================
// Metadata
// ============================================================================

/// Import plugin information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportPlugin {
    /// Plugin name that was used to import this fixture.
    pub plugin: NonEmptyString,

    /// Date when the fixture was imported.
    pub date: IsoDateString,

    /// Additional import comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<NonEmptyMultilineString>,
}

/// Fixture metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureMeta {
    /// Authors who created/modified this fixture definition.
    pub authors: Vec<NonEmptyString>,

    /// Date when the fixture was created.
    pub create_date: IsoDateString,

    /// Date of the last modification.
    pub last_modify_date: IsoDateString,

    /// Information about how the fixture was imported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_plugin: Option<ImportPlugin>,
}

// ============================================================================
// Links
// ============================================================================

/// Links to external resources.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureLinks {
    /// Links to product manuals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual: Option<Vec<UrlString>>,

    /// Links to product pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_page: Option<Vec<UrlString>>,

    /// Links to videos.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Vec<UrlString>>,

    /// Other relevant links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Vec<UrlString>>,
}

// ============================================================================
// RDM
// ============================================================================

/// RDM (Remote Device Management) information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureRdm {
    /// RDM model ID (0-65535).
    pub model_id: u16,

    /// Software version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<NonEmptyString>,
}

// ============================================================================
// Wheels
// ============================================================================

/// A wheel (color, gobo, etc.) definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wheel {
    /// Wheel rotation direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<WheelDirection>,

    /// Slots on the wheel.
    pub slots: Vec<WheelSlot>,
}

// ============================================================================
// Modes
// ============================================================================

/// A template channel key - can be null (unused), regular, or template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateChannelKey {
    Null,
    Template(VariablePixelKeyString),
}

/// Matrix channel insert block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixChannelInsert {
    /// Must be "matrixChannels".
    pub insert: MatrixChannelsMarker,

    /// How to repeat the channels.
    pub repeat_for: RepeatFor,

    /// Channel ordering.
    pub channel_order: ChannelOrder,

    /// Template channels to insert.
    pub template_channels: Vec<Option<VariablePixelKeyString>>,
}

/// Marker type for "matrixChannels" insert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MatrixChannelsMarker;

impl Serialize for MatrixChannelsMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("matrixChannels")
    }
}

impl<'de> Deserialize<'de> for MatrixChannelsMarker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "matrixChannels" {
            Ok(MatrixChannelsMarker)
        } else {
            Err(serde::de::Error::custom("expected 'matrixChannels'"))
        }
    }
}

/// A channel reference in a mode - can be null, a channel key, or a matrix insert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModeChannel {
    /// Unused channel slot.
    Null,
    /// Reference to a channel by key.
    Key(NoVariablesString),
    /// Matrix channel insert block.
    MatrixInsert(MatrixChannelInsert),
}

/// A fixture mode definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    /// Mode name (should not contain "mode" or "Mode").
    pub name: NonEmptyString,

    /// Short name for the mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NonEmptyString>,

    /// RDM personality index (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdm_personality_index: Option<u32>,

    /// Physical overrides for this mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical: Option<Physical>,

    /// Channels in this mode.
    pub channels: Vec<ModeChannel>,
}

// ============================================================================
// Fixture
// ============================================================================

/// Complete fixture definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    /// JSON Schema reference.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Fixture name (unique within manufacturer).
    pub name: NonEmptyString,

    /// Short name (globally unique).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<NonEmptyString>,

    /// Categories (most important first).
    pub categories: Vec<Category>,

    /// Fixture metadata.
    pub meta: FixtureMeta,

    /// General comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<NonEmptyMultilineString>,

    /// Links to external resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<FixtureLinks>,

    /// Request for help with this fixture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_wanted: Option<NonEmptyString>,

    /// RDM information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rdm: Option<FixtureRdm>,

    /// Physical properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical: Option<Physical>,

    /// Matrix definition for pixel-based fixtures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<Matrix>,

    /// Wheel definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wheels: Option<HashMap<NonEmptyString, Wheel>>,

    /// Available channels (keyed by channel key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_channels: Option<HashMap<NoVariablesString, Channel>>,

    /// Template channels for matrix fixtures (keyed by template channel key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_channels: Option<HashMap<VariablePixelKeyString, Channel>>,

    /// Fixture modes.
    pub modes: Vec<Mode>,
}

impl Fixture {
    /// Returns the primary category of this fixture.
    pub fn primary_category(&self) -> Option<Category> {
        self.categories.first().copied()
    }

    /// Returns the total number of DMX channels across all modes.
    pub fn max_channel_count(&self) -> usize {
        self.modes
            .iter()
            .map(|m| m.channels.len())
            .max()
            .unwrap_or(0)
    }

    /// Returns true if this fixture has a matrix.
    pub fn has_matrix(&self) -> bool {
        self.matrix.is_some()
    }

    /// Returns true if this fixture has RDM support.
    pub fn has_rdm(&self) -> bool {
        self.rdm.is_some()
    }
}
