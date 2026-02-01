#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

use serde::{Deserialize, Serialize};

/// ---- external definitions (placeholders) ----
/// Replace these with your actual domain types / newtypes.

pub type NonEmptyString = String;
pub type ColorString = String;
pub type GoboResourceString = String;

/// Percent types are usually constrained integers or floats.
/// JSON Schema 側の制約は型では表現できないため、
/// validation は別レイヤで行う前提。
pub type Percent = f64;
pub type IrisPercent = f64;
pub type ColorTemperature = u32;

/// ---- wheel-slot ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WheelSlot {
    Open,

    Closed,

    Color {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,

        #[serde(skip_serializing_if = "Option::is_none")]
        colors: Option<Vec<ColorString>>,

        #[serde(rename = "colorTemperature", skip_serializing_if = "Option::is_none")]
        color_temperature: Option<ColorTemperature>,
    },

    Gobo {
        #[serde(skip_serializing_if = "Option::is_none")]
        resource: Option<GoboResourceString>,

        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,
    },

    Prism {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,

        #[serde(skip_serializing_if = "Option::is_none")]
        facets: Option<u32>,
    },

    Iris {
        #[serde(rename = "openPercent", skip_serializing_if = "Option::is_none")]
        open_percent: Option<IrisPercent>,
    },

    Frost {
        #[serde(rename = "frostIntensity", skip_serializing_if = "Option::is_none")]
        frost_intensity: Option<Percent>,
    },

    AnimationGoboStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<NonEmptyString>,
    },

    AnimationGoboEnd,
}
