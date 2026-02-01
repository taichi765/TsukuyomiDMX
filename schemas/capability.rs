//! Capability types for DMX channel ranges.
//!
//! This module defines the various capability types that can be assigned to DMX value ranges
//! within a channel. Each capability type represents a specific function like strobe, color,
//! pan/tilt movement, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    BeamAngle, Brightness, ColorString, ColorTemperature, Distance, DmxValue, EffectPreset,
    FogOutput, HorizontalAngle, Insertion, IrisPercent, NonEmptyString, Parameter, Percent,
    RotationAngle, RotationSpeed, SlotNumber, Speed, SwingAngle, Time, VerticalAngle,
};

// ============================================================================
// Common Types
// ============================================================================

/// A DMX range represented as [start, end] values.
pub type DmxRange = [DmxValue; 2];

/// Menu click position for UI representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MenuClick {
    Start,
    Center,
    End,
    Hidden,
}

/// Switch channels mapping - maps switching channel alias keys to channel keys.
pub type SwitchChannels = HashMap<String, String>;

/// Color for ColorIntensity capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorIntensityColor {
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
    Amber,
    White,
    #[serde(rename = "Warm White")]
    WarmWhite,
    #[serde(rename = "Cold White")]
    ColdWhite,
    UV,
    Lime,
    Indigo,
}

/// Shutter effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShutterEffect {
    Open,
    Closed,
    Strobe,
    Pulse,
    RampUp,
    RampDown,
    RampUpDown,
    Lightning,
    Spikes,
    Burst,
}

/// Fog type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FogType {
    Fog,
    Haze,
}

/// What is shaking in WheelShake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShakingTarget {
    Wheel,
    Slot,
}

/// Blade identifier for blade capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Blade {
    Named(BladePosition),
    Numbered(u32),
}

/// Named blade positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BladePosition {
    Top,
    Right,
    Bottom,
    Left,
}

/// Wheel reference - can be a single wheel or multiple wheels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WheelRef {
    Single(NonEmptyString),
    Multiple(Vec<NonEmptyString>),
}

// ============================================================================
// Common Fields Struct (for sharing across capabilities)
// ============================================================================

/// Common optional fields present in most capabilities.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommonFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<NonEmptyString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_wanted: Option<NonEmptyString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_click: Option<MenuClick>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_channels: Option<SwitchChannels>,
}

// ============================================================================
// Capability Enum
// ============================================================================

/// A capability defines what a fixture does at a specific DMX value range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Capability {
    /// No function / unused range.
    NoFunction {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Shutter/strobe control.
    ShutterStrobe {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "shutterEffect")]
        shutter_effect: ShutterEffect,

        #[serde(rename = "soundControlled", skip_serializing_if = "Option::is_none")]
        sound_controlled: Option<bool>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<Time>,
        #[serde(rename = "durationStart", skip_serializing_if = "Option::is_none")]
        duration_start: Option<Time>,
        #[serde(rename = "durationEnd", skip_serializing_if = "Option::is_none")]
        duration_end: Option<Time>,

        #[serde(rename = "randomTiming", skip_serializing_if = "Option::is_none")]
        random_timing: Option<bool>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Strobe speed control.
    StrobeSpeed {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Strobe duration control.
    StrobeDuration {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<Time>,
        #[serde(rename = "durationStart", skip_serializing_if = "Option::is_none")]
        duration_start: Option<Time>,
        #[serde(rename = "durationEnd", skip_serializing_if = "Option::is_none")]
        duration_end: Option<Time>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// General intensity/dimmer control.
    Intensity {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        brightness: Option<Brightness>,
        #[serde(rename = "brightnessStart", skip_serializing_if = "Option::is_none")]
        brightness_start: Option<Brightness>,
        #[serde(rename = "brightnessEnd", skip_serializing_if = "Option::is_none")]
        brightness_end: Option<Brightness>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Single color intensity control.
    ColorIntensity {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        color: ColorIntensityColor,

        #[serde(skip_serializing_if = "Option::is_none")]
        brightness: Option<Brightness>,
        #[serde(rename = "brightnessStart", skip_serializing_if = "Option::is_none")]
        brightness_start: Option<Brightness>,
        #[serde(rename = "brightnessEnd", skip_serializing_if = "Option::is_none")]
        brightness_end: Option<Brightness>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Color preset selection.
    ColorPreset {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        colors: Option<Vec<ColorString>>,
        #[serde(rename = "colorsStart", skip_serializing_if = "Option::is_none")]
        colors_start: Option<Vec<ColorString>>,
        #[serde(rename = "colorsEnd", skip_serializing_if = "Option::is_none")]
        colors_end: Option<Vec<ColorString>>,

        #[serde(rename = "colorTemperature", skip_serializing_if = "Option::is_none")]
        color_temperature: Option<ColorTemperature>,
        #[serde(
            rename = "colorTemperatureStart",
            skip_serializing_if = "Option::is_none"
        )]
        color_temperature_start: Option<ColorTemperature>,
        #[serde(
            rename = "colorTemperatureEnd",
            skip_serializing_if = "Option::is_none"
        )]
        color_temperature_end: Option<ColorTemperature>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Color temperature control.
    ColorTemperature {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "colorTemperature", skip_serializing_if = "Option::is_none")]
        color_temperature: Option<ColorTemperature>,
        #[serde(
            rename = "colorTemperatureStart",
            skip_serializing_if = "Option::is_none"
        )]
        color_temperature_start: Option<ColorTemperature>,
        #[serde(
            rename = "colorTemperatureEnd",
            skip_serializing_if = "Option::is_none"
        )]
        color_temperature_end: Option<ColorTemperature>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Pan (horizontal) position control.
    Pan {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Continuous pan rotation.
    PanContinuous {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Tilt (vertical) position control.
    Tilt {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Continuous tilt rotation.
    TiltContinuous {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Pan/tilt movement speed.
    PanTiltSpeed {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<Time>,
        #[serde(rename = "durationStart", skip_serializing_if = "Option::is_none")]
        duration_start: Option<Time>,
        #[serde(rename = "durationEnd", skip_serializing_if = "Option::is_none")]
        duration_end: Option<Time>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Wheel slot selection (gobo, color, etc.).
    WheelSlot {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        wheel: Option<NonEmptyString>,

        #[serde(rename = "slotNumber", skip_serializing_if = "Option::is_none")]
        slot_number: Option<SlotNumber>,
        #[serde(rename = "slotNumberStart", skip_serializing_if = "Option::is_none")]
        slot_number_start: Option<SlotNumber>,
        #[serde(rename = "slotNumberEnd", skip_serializing_if = "Option::is_none")]
        slot_number_end: Option<SlotNumber>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Wheel shake effect.
    WheelShake {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        wheel: Option<WheelRef>,

        #[serde(rename = "isShaking", skip_serializing_if = "Option::is_none")]
        is_shaking: Option<ShakingTarget>,

        #[serde(rename = "slotNumber", skip_serializing_if = "Option::is_none")]
        slot_number: Option<SlotNumber>,
        #[serde(rename = "slotNumberStart", skip_serializing_if = "Option::is_none")]
        slot_number_start: Option<SlotNumber>,
        #[serde(rename = "slotNumberEnd", skip_serializing_if = "Option::is_none")]
        slot_number_end: Option<SlotNumber>,

        #[serde(rename = "shakeSpeed", skip_serializing_if = "Option::is_none")]
        shake_speed: Option<Speed>,
        #[serde(rename = "shakeSpeedStart", skip_serializing_if = "Option::is_none")]
        shake_speed_start: Option<Speed>,
        #[serde(rename = "shakeSpeedEnd", skip_serializing_if = "Option::is_none")]
        shake_speed_end: Option<Speed>,

        #[serde(rename = "shakeAngle", skip_serializing_if = "Option::is_none")]
        shake_angle: Option<SwingAngle>,
        #[serde(rename = "shakeAngleStart", skip_serializing_if = "Option::is_none")]
        shake_angle_start: Option<SwingAngle>,
        #[serde(rename = "shakeAngleEnd", skip_serializing_if = "Option::is_none")]
        shake_angle_end: Option<SwingAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Wheel slot rotation (gobo rotation).
    WheelSlotRotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        wheel: Option<WheelRef>,

        #[serde(rename = "slotNumber", skip_serializing_if = "Option::is_none")]
        slot_number: Option<SlotNumber>,
        #[serde(rename = "slotNumberStart", skip_serializing_if = "Option::is_none")]
        slot_number_start: Option<SlotNumber>,
        #[serde(rename = "slotNumberEnd", skip_serializing_if = "Option::is_none")]
        slot_number_end: Option<SlotNumber>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Wheel rotation (spinning the whole wheel).
    WheelRotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        wheel: Option<WheelRef>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Generic effect.
    Effect {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "effectName", skip_serializing_if = "Option::is_none")]
        effect_name: Option<NonEmptyString>,
        #[serde(rename = "effectPreset", skip_serializing_if = "Option::is_none")]
        effect_preset: Option<EffectPreset>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<Time>,
        #[serde(rename = "durationStart", skip_serializing_if = "Option::is_none")]
        duration_start: Option<Time>,
        #[serde(rename = "durationEnd", skip_serializing_if = "Option::is_none")]
        duration_end: Option<Time>,

        #[serde(skip_serializing_if = "Option::is_none")]
        parameter: Option<Parameter>,
        #[serde(rename = "parameterStart", skip_serializing_if = "Option::is_none")]
        parameter_start: Option<Parameter>,
        #[serde(rename = "parameterEnd", skip_serializing_if = "Option::is_none")]
        parameter_end: Option<Parameter>,

        #[serde(rename = "soundControlled", skip_serializing_if = "Option::is_none")]
        sound_controlled: Option<bool>,

        #[serde(rename = "soundSensitivity", skip_serializing_if = "Option::is_none")]
        sound_sensitivity: Option<Percent>,
        #[serde(
            rename = "soundSensitivityStart",
            skip_serializing_if = "Option::is_none"
        )]
        sound_sensitivity_start: Option<Percent>,
        #[serde(
            rename = "soundSensitivityEnd",
            skip_serializing_if = "Option::is_none"
        )]
        sound_sensitivity_end: Option<Percent>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Effect speed control.
    EffectSpeed {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Effect duration control.
    EffectDuration {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<Time>,
        #[serde(rename = "durationStart", skip_serializing_if = "Option::is_none")]
        duration_start: Option<Time>,
        #[serde(rename = "durationEnd", skip_serializing_if = "Option::is_none")]
        duration_end: Option<Time>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Effect parameter control.
    EffectParameter {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        parameter: Option<Parameter>,
        #[serde(rename = "parameterStart", skip_serializing_if = "Option::is_none")]
        parameter_start: Option<Parameter>,
        #[serde(rename = "parameterEnd", skip_serializing_if = "Option::is_none")]
        parameter_end: Option<Parameter>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Sound sensitivity control.
    SoundSensitivity {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "soundSensitivity", skip_serializing_if = "Option::is_none")]
        sound_sensitivity: Option<Percent>,
        #[serde(
            rename = "soundSensitivityStart",
            skip_serializing_if = "Option::is_none"
        )]
        sound_sensitivity_start: Option<Percent>,
        #[serde(
            rename = "soundSensitivityEnd",
            skip_serializing_if = "Option::is_none"
        )]
        sound_sensitivity_end: Option<Percent>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Beam angle control.
    BeamAngle {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<BeamAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<BeamAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<BeamAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Beam position control.
    BeamPosition {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "horizontalAngle", skip_serializing_if = "Option::is_none")]
        horizontal_angle: Option<HorizontalAngle>,
        #[serde(
            rename = "horizontalAngleStart",
            skip_serializing_if = "Option::is_none"
        )]
        horizontal_angle_start: Option<HorizontalAngle>,
        #[serde(rename = "horizontalAngleEnd", skip_serializing_if = "Option::is_none")]
        horizontal_angle_end: Option<HorizontalAngle>,

        #[serde(rename = "verticalAngle", skip_serializing_if = "Option::is_none")]
        vertical_angle: Option<VerticalAngle>,
        #[serde(rename = "verticalAngleStart", skip_serializing_if = "Option::is_none")]
        vertical_angle_start: Option<VerticalAngle>,
        #[serde(rename = "verticalAngleEnd", skip_serializing_if = "Option::is_none")]
        vertical_angle_end: Option<VerticalAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Focus control.
    Focus {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        distance: Option<Distance>,
        #[serde(rename = "distanceStart", skip_serializing_if = "Option::is_none")]
        distance_start: Option<Distance>,
        #[serde(rename = "distanceEnd", skip_serializing_if = "Option::is_none")]
        distance_end: Option<Distance>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Zoom control.
    Zoom {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<BeamAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<BeamAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<BeamAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Iris control.
    Iris {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "openPercent", skip_serializing_if = "Option::is_none")]
        open_percent: Option<IrisPercent>,
        #[serde(rename = "openPercentStart", skip_serializing_if = "Option::is_none")]
        open_percent_start: Option<IrisPercent>,
        #[serde(rename = "openPercentEnd", skip_serializing_if = "Option::is_none")]
        open_percent_end: Option<IrisPercent>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Iris effect.
    IrisEffect {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "effectName")]
        effect_name: NonEmptyString,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Frost filter control.
    Frost {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "frostIntensity", skip_serializing_if = "Option::is_none")]
        frost_intensity: Option<Percent>,
        #[serde(
            rename = "frostIntensityStart",
            skip_serializing_if = "Option::is_none"
        )]
        frost_intensity_start: Option<Percent>,
        #[serde(rename = "frostIntensityEnd", skip_serializing_if = "Option::is_none")]
        frost_intensity_end: Option<Percent>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Frost effect.
    FrostEffect {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "effectName")]
        effect_name: NonEmptyString,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Prism insertion/control.
    Prism {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Prism rotation control.
    PrismRotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Blade insertion control.
    BladeInsertion {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        blade: Blade,

        #[serde(skip_serializing_if = "Option::is_none")]
        insertion: Option<Insertion>,
        #[serde(rename = "insertionStart", skip_serializing_if = "Option::is_none")]
        insertion_start: Option<Insertion>,
        #[serde(rename = "insertionEnd", skip_serializing_if = "Option::is_none")]
        insertion_end: Option<Insertion>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Blade rotation control.
    BladeRotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        blade: Blade,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Blade system rotation control.
    BladeSystemRotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Fog machine control.
    Fog {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "fogType", skip_serializing_if = "Option::is_none")]
        fog_type: Option<FogType>,

        #[serde(rename = "fogOutput", skip_serializing_if = "Option::is_none")]
        fog_output: Option<FogOutput>,
        #[serde(rename = "fogOutputStart", skip_serializing_if = "Option::is_none")]
        fog_output_start: Option<FogOutput>,
        #[serde(rename = "fogOutputEnd", skip_serializing_if = "Option::is_none")]
        fog_output_end: Option<FogOutput>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Fog output control.
    FogOutput {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "fogOutput", skip_serializing_if = "Option::is_none")]
        fog_output: Option<FogOutput>,
        #[serde(rename = "fogOutputStart", skip_serializing_if = "Option::is_none")]
        fog_output_start: Option<FogOutput>,
        #[serde(rename = "fogOutputEnd", skip_serializing_if = "Option::is_none")]
        fog_output_end: Option<FogOutput>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Fog type selection.
    FogType {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(rename = "fogType")]
        fog_type: FogType,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Generic rotation control.
    Rotation {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<RotationSpeed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<RotationSpeed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<RotationSpeed>,

        #[serde(skip_serializing_if = "Option::is_none")]
        angle: Option<RotationAngle>,
        #[serde(rename = "angleStart", skip_serializing_if = "Option::is_none")]
        angle_start: Option<RotationAngle>,
        #[serde(rename = "angleEnd", skip_serializing_if = "Option::is_none")]
        angle_end: Option<RotationAngle>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Generic speed control.
    Speed {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<Speed>,
        #[serde(rename = "speedStart", skip_serializing_if = "Option::is_none")]
        speed_start: Option<Speed>,
        #[serde(rename = "speedEnd", skip_serializing_if = "Option::is_none")]
        speed_end: Option<Speed>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Generic time control.
    Time {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        time: Option<Time>,
        #[serde(rename = "timeStart", skip_serializing_if = "Option::is_none")]
        time_start: Option<Time>,
        #[serde(rename = "timeEnd", skip_serializing_if = "Option::is_none")]
        time_end: Option<Time>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Maintenance functions (lamp on/off, reset, etc.).
    Maintenance {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(skip_serializing_if = "Option::is_none")]
        parameter: Option<Parameter>,
        #[serde(rename = "parameterStart", skip_serializing_if = "Option::is_none")]
        parameter_start: Option<Parameter>,
        #[serde(rename = "parameterEnd", skip_serializing_if = "Option::is_none")]
        parameter_end: Option<Parameter>,

        #[serde(skip_serializing_if = "Option::is_none")]
        hold: Option<Time>,

        #[serde(flatten)]
        common: CommonFields,
    },

    /// Generic/catch-all capability.
    Generic {
        #[serde(rename = "dmxRange", skip_serializing_if = "Option::is_none")]
        dmx_range: Option<DmxRange>,

        #[serde(flatten)]
        common: CommonFields,
    },
}
