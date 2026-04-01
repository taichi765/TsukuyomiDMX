//! Open Fixture Library Schema Types
//!
//! This crate provides Rust types for the Open Fixture Library JSON schemas.
//! These types can be used to parse, validate, and manipulate fixture definitions.
//!
//! # Main Types
//!
//! - [`Fixture`] - Complete fixture definition
//! - [`Channel`] - DMX channel definition
//! - [`Capability`] - Channel capability (what a DMX range does)
//! - [`Matrix`] - Pixel matrix definition for LED fixtures
//! - [`Manufacturers`] - Collection of manufacturer definitions
//! - [`Gobo`] - Gobo pattern definition
//! - [`WheelSlot`] - Wheel slot (color/gobo/prism) definition
//! - [`Plugin`] - Import/export plugin definition
//! - [`FixtureRedirect`] - Fixture redirect for renamed/aliased fixtures
//!
//! # Example
//!
//! ```ignore
//! use schemas::{Fixture, Category};
//!
//! let json = std::fs::read_to_string("fixture.json")?;
//! let fixture: Fixture = serde_json::from_str(&json)?;
//!
//! println!("Fixture: {}", fixture.name);
//! println!("Categories: {:?}", fixture.categories);
//! println!("Modes: {}", fixture.modes.len());
//! ```

// Module declarations
mod capability;
mod channel;
mod definitions;
mod fixture;
mod fixture_redirect;
mod gobo;
mod manufacturers;
mod matrix;
mod plugin;
mod wheel_slot;

// Re-export all public types from each module

// definitions.rs - Basic types
pub use definitions::{
    BeamAngle,
    BeamAngleKeyword,
    BeatsPerMinute,
    Brightness,
    BrightnessKeyword,
    ColorString,
    ColorTemperature,
    ColorTemperatureKeyword,
    Degrees,
    // Dimension types
    DimensionsXYZ,
    Distance,
    DistanceKeyword,
    DmxValue,
    DmxValuePercent,
    // Enums
    EffectPreset,
    FogOutput,
    FogOutputKeyword,
    GoboResourceString,
    Hertz,
    HorizontalAngle,
    HorizontalAngleKeyword,
    Insertion,
    InsertionKeyword,
    IrisPercent,
    IrisPercentKeyword,
    IsoDateString,
    Kelvin,
    Lumens,
    Meters,
    MilliSeconds,
    ModeNameString,
    NoVariablesString,
    NonEmptyMultilineString,
    // String types
    NonEmptyString,
    NonNegativeNumber,
    // Unit types
    Number,
    Parameter,
    ParameterKeyword,
    Percent,
    PercentKeyword,
    PercentString,
    PositiveInteger,
    PowerConnectorType,
    RotationAngle,
    RotationSpeed,
    RotationSpeedKeyword,
    RoundsPerMinute,
    Seconds,
    SlotNumber,
    // Entity types
    Speed,
    SpeedKeyword,
    SwingAngle,
    SwingAngleKeyword,
    Time,
    TimeKeyword,
    UrlArray,
    UrlString,
    VariablePixelKeyString,
    VerticalAngle,
    VerticalAngleKeyword,
    VolumePerMinute,
};

// capability.rs - Capability types
pub use capability::{
    Blade, BladePosition, Capability, ColorIntensityColor, CommonFields, DmxRange, FogType,
    MenuClick, ShakingTarget, ShutterEffect, SwitchChannels, WheelRef,
};

// channel.rs - Channel types
pub use channel::{Channel, DmxValueOrPercent, DmxValueResolution, FineChannelAlias, Precedence};

// matrix.rs - Matrix types
pub use matrix::{
    AllPixels, DivisiblePosition, DivisibleWithRemainderPosition, EvenPosition, ExactPosition,
    Matrix, MaximumPosition, MinimumPosition, OddPosition, PixelConstraints, PixelGroup, PixelKey,
    PixelNumberConstraint,
};

// fixture.rs - Fixture types
pub use fixture::{
    Bulb, Category, ChannelOrder, DmxConnector, Fixture, FixtureLinks, FixtureMeta, FixtureRdm,
    ImportPlugin, Lens, MatrixChannelInsert, MatrixChannelsMarker, MatrixPixels, Mode, ModeChannel,
    Physical, PowerConnectorMode, PowerConnectors, RepeatFor, RepeatOrder, TemplateChannelKey,
    Wheel, WheelDirection,
};

// manufacturers.rs - Manufacturer types
pub use manufacturers::{Manufacturer, Manufacturers};

// gobo.rs - Gobo types
pub use gobo::{Gobo, GoboSource};

// wheel_slot.rs - Wheel slot types
pub use wheel_slot::WheelSlot;

// plugin.rs - Plugin types
pub use plugin::{FileLocations, HtmlStringLines, PlatformFileLocations, Plugin};

// fixture_redirect.rs - Fixture redirect types
pub use fixture_redirect::{FixtureRedirect, RedirectReason};

#[cfg(test)]
mod tests {
    use super::*;

    const CAMEO_AURO_SPOT_300: &str = include_str!("test_fixtures/cameo_auro_spot_300.json");
    const ADJ_MEGA_TRIPAR: &str = include_str!("test_fixtures/adj_mega_tripar.json");

    #[test]
    fn test_parse_cameo_auro_spot_300() {
        let fixture: Fixture =
            serde_json::from_str(CAMEO_AURO_SPOT_300).expect("Failed to parse Cameo Auro Spot 300");

        assert_eq!(fixture.name, "Auro Spot 300");
        assert_eq!(fixture.short_name, Some("CLAS300".to_string()));
        assert_eq!(fixture.categories.len(), 2);
        assert!(fixture.categories.contains(&Category::MovingHead));
        assert!(fixture.categories.contains(&Category::ColorChanger));

        // Check meta
        assert_eq!(fixture.meta.authors, vec!["Felix Edelmann"]);
        assert_eq!(fixture.meta.create_date, "2019-02-08");
        assert_eq!(fixture.meta.last_modify_date, "2019-02-08");

        // Check physical
        let physical = fixture.physical.as_ref().expect("Expected physical");
        assert_eq!(physical.dimensions, Some([285.0, 485.0, 180.0]));
        assert_eq!(physical.weight, Some(8.75));
        assert_eq!(physical.power, Some(320.0));

        // Check wheels
        let wheels = fixture.wheels.as_ref().expect("Expected wheels");
        assert!(wheels.contains_key("Color Wheel"));
        assert!(wheels.contains_key("Gobo Wheel 1"));
        assert!(wheels.contains_key("Gobo Wheel 2"));

        // Check available channels
        let channels = fixture
            .available_channels
            .as_ref()
            .expect("Expected available_channels");
        assert!(channels.contains_key("Pan"));
        assert!(channels.contains_key("Tilt"));
        assert!(channels.contains_key("Dimmer"));
        assert!(channels.contains_key("Strobe"));

        // Check modes
        assert_eq!(fixture.modes.len(), 3);
        assert_eq!(fixture.modes[0].name, "5-channel");
        assert_eq!(fixture.modes[1].name, "15-channel");
        assert_eq!(fixture.modes[2].name, "24-channel");
    }

    #[test]
    fn test_parse_adj_mega_tripar() {
        let fixture: Fixture =
            serde_json::from_str(ADJ_MEGA_TRIPAR).expect("Failed to parse ADJ Mega TRIPAR");

        assert_eq!(fixture.name, "Mega TRIPAR Profile Plus");
        assert_eq!(fixture.categories.len(), 1);
        assert!(fixture.categories.contains(&Category::ColorChanger));

        // Check channels with switchChannels
        let channels = fixture
            .available_channels
            .as_ref()
            .expect("Expected available_channels");
        assert!(channels.contains_key("Program Selection Mode"));
        assert!(channels.contains_key("Color Macros"));
        assert!(channels.contains_key("Sound Sensitivity"));

        // Check modes
        assert_eq!(fixture.modes.len(), 2);
        assert_eq!(fixture.modes[0].name, "4 channel");
        assert_eq!(fixture.modes[1].name, "9 channel");
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Parse the fixture
        let fixture: Fixture =
            serde_json::from_str(CAMEO_AURO_SPOT_300).expect("Failed to parse fixture");

        // Serialize back to JSON
        let serialized =
            serde_json::to_string_pretty(&fixture).expect("Failed to serialize fixture");

        // Parse the serialized JSON
        let reparsed: Fixture =
            serde_json::from_str(&serialized).expect("Failed to reparse fixture");

        // Verify key fields match
        assert_eq!(fixture.name, reparsed.name);
        assert_eq!(fixture.short_name, reparsed.short_name);
        assert_eq!(fixture.categories, reparsed.categories);
        assert_eq!(fixture.modes.len(), reparsed.modes.len());

        // Check that all mode names match
        for (orig, repr) in fixture.modes.iter().zip(reparsed.modes.iter()) {
            assert_eq!(orig.name, repr.name);
            assert_eq!(orig.short_name, repr.short_name);
        }
    }
}
