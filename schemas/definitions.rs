//! Basic type definitions referenced by other schemas.
//!
//! This module provides foundational types like string patterns, units, and entity types
//! that are used throughout the Open Fixture Library schema.

use serde::{Deserialize, Serialize};

// ============================================================================
// String Types
// ============================================================================

/// A non-empty string that cannot contain newlines.
/// Pattern: `^[^\n]+$`
pub type NonEmptyString = String;

/// A string that cannot contain `$` or newlines (no template variables).
/// Pattern: `^[^$\n]+$`
pub type NoVariablesString = String;

/// A string that must contain `$pixelKey` (template variable).
/// Pattern: `\$pixelKey`
pub type VariablePixelKeyString = String;

/// A non-empty string that can span multiple lines.
pub type NonEmptyMultilineString = String;

/// A mode name string that cannot contain "mode" or "Mode".
/// Pattern: `^((?!mode)(?!Mode).)*$`
pub type ModeNameString = String;

/// A URL string with http, https, or ftp scheme.
/// Pattern: `^(ftp|http|https)://[^ "]+$`
pub type UrlString = String;

/// An array of unique URL strings.
pub type UrlArray = Vec<UrlString>;

/// An ISO date string in YYYY-MM-DD format.
/// Pattern: `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`
pub type IsoDateString = String;

/// A hex color string.
/// Pattern: `^#[0-9a-f]{6}$`
pub type ColorString = String;

/// A gobo resource path.
/// Pattern: `^gobos/[a-z0-9-]+$|^gobos/aliases/[a-z0-9_.-]+/`
pub type GoboResourceString = String;

// ============================================================================
// Dimension Types
// ============================================================================

/// Dimensions in X, Y, Z order (width, height, depth in mm).
pub type DimensionsXYZ = [f64; 3];

// ============================================================================
// Enums
// ============================================================================

/// Effect preset types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectPreset {
    ColorJump,
    ColorFade,
}

/// Power connector types indicating input/output capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerConnectorType {
    #[serde(rename = "input only")]
    InputOnly,
    #[serde(rename = "output only")]
    OutputOnly,
    #[serde(rename = "input and output")]
    InputAndOutput,
}

// ============================================================================
// Unit Types
// ============================================================================

/// A numeric value (any number).
pub type Number = f64;

/// A non-negative number (>= 0).
pub type NonNegativeNumber = f64;

/// A positive integer (> 0).
pub type PositiveInteger = u32;

/// A DMX value (0-255 for 8-bit, up to 65535 for 16-bit, etc.).
pub type DmxValue = u32;

/// A DMX value as percentage string.
/// Pattern: `^(([1-9][0-9]?|0)(\.[0-9]+)?|100)%$`
pub type DmxValuePercent = String;

/// A percentage value string (can be negative).
/// Pattern: `^-?[0-9]+(\.[0-9]+)?%$`
pub type PercentString = String;

/// A frequency in Hertz.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?Hz$`
pub type Hertz = String;

/// Beats per minute.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?bpm$`
pub type BeatsPerMinute = String;

/// Rounds (rotations) per minute.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?rpm$`
pub type RoundsPerMinute = String;

/// Time in seconds.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?s$`
pub type Seconds = String;

/// Time in milliseconds.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?ms$`
pub type MilliSeconds = String;

/// Distance in meters.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?m$`
pub type Meters = String;

/// Luminous flux in lumens.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?lm$`
pub type Lumens = String;

/// Color temperature in Kelvin.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?K$`
pub type Kelvin = String;

/// Volume flow rate (m³/min).
/// Pattern: `^-?[0-9]+(\.[0-9]+)?m\^3/min$`
pub type VolumePerMinute = String;

/// Angle in degrees.
/// Pattern: `^-?[0-9]+(\.[0-9]+)?deg$`
pub type Degrees = String;

// ============================================================================
// Entity Types (union types for capability values)
// ============================================================================

/// Speed entity - can be Hz, bpm, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Speed {
    Hertz(Hertz),
    Bpm(BeatsPerMinute),
    Percent(PercentString),
    Keyword(SpeedKeyword),
}

/// Speed keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeedKeyword {
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "slow")]
    Slow,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "slow reverse")]
    SlowReverse,
    #[serde(rename = "fast reverse")]
    FastReverse,
}

/// Rotation speed entity - can be Hz, rpm, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RotationSpeed {
    Hertz(Hertz),
    Rpm(RoundsPerMinute),
    Percent(PercentString),
    Keyword(RotationSpeedKeyword),
}

/// Rotation speed keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationSpeedKeyword {
    #[serde(rename = "fast CW")]
    FastCw,
    #[serde(rename = "slow CW")]
    SlowCw,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "slow CCW")]
    SlowCcw,
    #[serde(rename = "fast CCW")]
    FastCcw,
}

/// Time entity - can be seconds, milliseconds, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Time {
    Seconds(Seconds),
    MilliSeconds(MilliSeconds),
    Percent(PercentString),
    Keyword(TimeKeyword),
}

/// Time keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeKeyword {
    #[serde(rename = "instant")]
    Instant,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "long")]
    Long,
}

/// Distance entity - can be meters, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Distance {
    Meters(Meters),
    Percent(PercentString),
    Keyword(DistanceKeyword),
}

/// Distance keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceKeyword {
    #[serde(rename = "near")]
    Near,
    #[serde(rename = "far")]
    Far,
}

/// Brightness entity - can be lumens, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Brightness {
    Lumens(Lumens),
    Percent(PercentString),
    Keyword(BrightnessKeyword),
}

/// Brightness keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrightnessKeyword {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "bright")]
    Bright,
}

/// Color temperature entity - can be Kelvin, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorTemperature {
    Kelvin(Kelvin),
    Percent(PercentString),
    Keyword(ColorTemperatureKeyword),
}

/// Color temperature keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorTemperatureKeyword {
    #[serde(rename = "warm")]
    Warm,
    #[serde(rename = "CTO")]
    Cto,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "cold")]
    Cold,
    #[serde(rename = "CTB")]
    Ctb,
}

/// Fog output entity - can be volume/min, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FogOutput {
    VolumePerMinute(VolumePerMinute),
    Percent(PercentString),
    Keyword(FogOutputKeyword),
}

/// Fog output keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FogOutputKeyword {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "weak")]
    Weak,
    #[serde(rename = "strong")]
    Strong,
}

/// Rotation angle entity - can be degrees or percent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RotationAngle {
    Degrees(Degrees),
    Percent(PercentString),
}

/// Beam angle entity - can be degrees, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BeamAngle {
    Degrees(Degrees),
    Percent(PercentString),
    Keyword(BeamAngleKeyword),
}

/// Beam angle keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeamAngleKeyword {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "narrow")]
    Narrow,
    #[serde(rename = "wide")]
    Wide,
}

/// Horizontal angle entity - can be degrees, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HorizontalAngle {
    Degrees(Degrees),
    Percent(PercentString),
    Keyword(HorizontalAngleKeyword),
}

/// Horizontal angle keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAngleKeyword {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "right")]
    Right,
}

/// Vertical angle entity - can be degrees, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerticalAngle {
    Degrees(Degrees),
    Percent(PercentString),
    Keyword(VerticalAngleKeyword),
}

/// Vertical angle keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAngleKeyword {
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "bottom")]
    Bottom,
}

/// Swing angle entity - can be degrees, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SwingAngle {
    Degrees(Degrees),
    Percent(PercentString),
    Keyword(SwingAngleKeyword),
}

/// Swing angle keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwingAngleKeyword {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "narrow")]
    Narrow,
    #[serde(rename = "wide")]
    Wide,
}

/// Generic parameter entity - can be number, percent, or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Parameter {
    Number(Number),
    Percent(PercentString),
    Keyword(ParameterKeyword),
}

/// Parameter keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterKeyword {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "slow")]
    Slow,
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "small")]
    Small,
    #[serde(rename = "big")]
    Big,
    #[serde(rename = "instant")]
    Instant,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "long")]
    Long,
}

/// Slot number entity - a non-negative number.
pub type SlotNumber = NonNegativeNumber;

/// Percent entity - can be percent string or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Percent {
    Percent(PercentString),
    Keyword(PercentKeyword),
}

/// Percent keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentKeyword {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "high")]
    High,
}

/// Insertion entity - can be percent or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Insertion {
    Percent(PercentString),
    Keyword(InsertionKeyword),
}

/// Insertion keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsertionKeyword {
    #[serde(rename = "out")]
    Out,
    #[serde(rename = "in")]
    In,
}

/// Iris percent entity - can be percent or keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IrisPercent {
    Percent(PercentString),
    Keyword(IrisPercentKeyword),
}

/// Iris percent keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrisPercentKeyword {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "open")]
    Open,
}
