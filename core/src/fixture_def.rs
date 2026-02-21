use std::{collections::HashMap, sync::Arc};

use bimap::BiHashMap;
use thiserror::Error;

use crate::{fixture::MergeMode, prelude::DmxAddress};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FixtureDefId(Arc<(ofl_schemas::NonEmptyString, ofl_schemas::NonEmptyString)>);

impl FixtureDefId {
    /// The API may change in future.
    /// We currently use manufacturer and model as an id, but Uuid might be better.
    pub fn new(
        manufacturer: ofl_schemas::NonEmptyString,
        model: ofl_schemas::NonEmptyString,
    ) -> Self {
        Self(Arc::new((manufacturer, model)))
    }
}

impl Clone for FixtureDefId {
    /// Cheap clone (Same as [`Arc::clone()`][std::sync::Arc])
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct FixtureDef {
    id: FixtureDefId,
    manufacturer: String,
    model: String,
    channel_templates: HashMap<String, ChannelDef>,
    modes: HashMap<String, FixtureMode>,
}

impl FixtureDef {
    // TODO: すべての関数でimpl Into<String>を使うようにする
    pub fn new(manufacturer: impl Into<String>, model: impl Into<String>) -> Self {
        let manufacturer = manufacturer.into();
        let model = model.into();
        Self {
            id: FixtureDefId::new(manufacturer.clone(), model.clone()),
            manufacturer,
            model,
            modes: HashMap::new(),
            channel_templates: HashMap::new(),
        }
    }

    pub fn id(&self) -> &FixtureDefId {
        &self.id
    }

    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn mode(&self, name: &str) -> Option<&FixtureMode> {
        self.modes.get(name)
    }

    pub fn channel_template(&self, name: &str) -> Option<&ChannelDef> {
        self.channel_templates.get(name)
    }

    // TODO: バリデーション
    /// Same as [std::collections::HashMap::insert()]
    pub fn insert_mode(
        &mut self,
        name: impl Into<String>,
        mode: FixtureMode,
    ) -> Option<FixtureMode> {
        self.modes.insert(name.into(), mode)
    }

    /// Same as [std::collections::HashMap::insert()]
    pub fn insert_channel(
        &mut self,
        name: impl Into<String>,
        channel: ChannelDef,
    ) -> Option<ChannelDef> {
        self.channel_templates.insert(name.into(), channel)
    }
}

impl TryFrom<ofl_schemas::Fixture> for FixtureDef {
    type Error = FixtureDefConverseError;
    fn try_from(_value: ofl_schemas::Fixture) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum FixtureDefConverseError {}

#[derive(Debug, Error)]
pub enum FixtureModeCreateError {
    #[error("{} offset are duplicated",.duplicates.len())]
    Duplicated { duplicates: Vec<DuplicatedError> },
    #[error("Mode must include at least 1 channel")]
    Empty,
    #[error("channel order was not contiguous")]
    NotContiguous,
}

#[derive(Debug, Error)]
pub enum DuplicatedError {
    #[error("offset {offset} is used by these channels: {channels:?}")]
    OffsetDuplicated {
        offset: usize,
        channels: Vec<String>,
    },
    #[error("channel {channel} is used by these offsets: {offsets:?}")]
    ChannelDuplicated {
        channel: String,
        offsets: Vec<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct FixtureMode {
    channel_order: BiHashMap<String, usize>,
}

impl FixtureMode {
    /// Creates new `FixtureMode`.
    pub fn new(
        channel_order: impl Iterator<Item = (String, usize)>,
    ) -> Result<Self, FixtureModeCreateError> {
        let mut map = BiHashMap::new();
        let mut ch_duplicates: HashMap<String, Vec<usize>> = HashMap::new();
        let mut off_duplicates: HashMap<usize, Vec<String>> = HashMap::new();
        for (ch, off) in channel_order {
            if let Some(first_off) = map.get_by_left(&ch) {
                ch_duplicates
                    .entry(ch)
                    .and_modify(|v| v.push(off))
                    .or_insert(vec![*first_off, off]);
                continue;
            }

            if let Some(first_ch) = map.get_by_right(&off).cloned() {
                off_duplicates
                    .entry(off)
                    .and_modify(|v| v.push(ch.clone())) // TODO: clone
                    .or_insert(vec![first_ch, ch]);
                continue;
            }

            map.insert_no_overwrite(ch, off).expect("logic error");
        }

        let ch_errors = ch_duplicates
            .into_iter()
            .map(|(channel, offsets)| DuplicatedError::ChannelDuplicated { channel, offsets });
        let offset_errors = off_duplicates
            .into_iter()
            .map(|(offset, channels)| DuplicatedError::OffsetDuplicated { offset, channels });
        let errors: Vec<DuplicatedError> = ch_errors.chain(offset_errors).collect();

        if !errors.is_empty() {
            return Err(FixtureModeCreateError::Duplicated { duplicates: errors });
        }

        let Some(max) = map.right_values().copied().max() else {
            return Err(FixtureModeCreateError::Empty);
        };
        if max != map.len() - 1 {
            return Err(FixtureModeCreateError::NotContiguous);
        }

        Ok(Self { channel_order: map })
    }

    /// Total length of the channels in this mode.
    pub fn footprint(&self) -> usize {
        self.channel_order.len()
    }

    pub fn occupied_addresses(
        &self,
        start_address: DmxAddress,
    ) -> impl Iterator<Item = DmxAddress> {
        AddressIter::new(start_address, self.footprint())
    }

    pub fn get_offset_by_channel(&self, channel: &str) -> Option<usize> {
        self.channel_order.get_by_left(channel).map(|n| *n)
    }

    pub fn get_channel_by_offset(&self, offset: usize) -> Option<&str> {
        self.channel_order.get_by_right(&offset).map(|s| s.as_str())
    }
}

pub struct AddressIter {
    start_address: DmxAddress,
    footprint: usize,
    count: usize,
}

impl AddressIter {
    fn new(start_address: DmxAddress, footprint: usize) -> Self {
        Self {
            start_address,
            footprint,
            count: 0,
        }
    }
}

impl Iterator for AddressIter {
    type Item = DmxAddress;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.footprint {
            None
        } else {
            let ret = self
                .start_address
                .checked_add(self.count)
                .expect("これはsafe?");
            self.count += 1;
            Some(ret)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelDef {
    merge_mode: MergeMode,
    kind: ChannelKind,
}

impl ChannelDef {
    pub fn new(merge_mode: MergeMode, kind: ChannelKind) -> Self {
        Self { merge_mode, kind }
    }

    pub fn merge_mode(&self) -> MergeMode {
        self.merge_mode
    }

    pub fn kind(&self) -> &ChannelKind {
        &self.kind
    }
}

// TODO: Add more kinds
#[derive(Debug, Clone)]
pub enum ChannelKind {
    Dimmer,
    Red,
    Blue,
    Green,
    White,
    WarmWhite,
    ColdWhite,
    Amber,
    UV,
    Custom, // TODO: open-fixture-library互換にする
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixture_mode_new {
        use super::*;

        #[test]
        fn creates_mode_with_single_channel() {
            let channels = vec![("Dimmer".to_string(), 0)];

            let mode = FixtureMode::new(channels.into_iter()).unwrap();

            assert_eq!(mode.footprint(), 1);
            assert_eq!(mode.get_offset_by_channel("Dimmer"), Some(0));
        }

        #[test]
        fn creates_mode_with_valid_channel_order() {
            let channels = vec![
                ("Dimmer".to_string(), 0),
                ("Red".to_string(), 1),
                ("Green".to_string(), 2),
                ("Blue".to_string(), 3),
            ];

            let mode = FixtureMode::new(channels.into_iter()).unwrap();

            assert_eq!(mode.footprint(), 4);
            assert_eq!(mode.get_offset_by_channel("Dimmer"), Some(0));
            assert_eq!(mode.get_offset_by_channel("Red"), Some(1));
            assert_eq!(mode.get_offset_by_channel("Green"), Some(2));
            assert_eq!(mode.get_offset_by_channel("Blue"), Some(3));
            assert_eq!(mode.get_channel_by_offset(0), Some("Dimmer"));
            assert_eq!(mode.get_channel_by_offset(1), Some("Red"));
            assert_eq!(mode.get_channel_by_offset(2), Some("Green"));
            assert_eq!(mode.get_channel_by_offset(3), Some("Blue"));
        }

        #[test]
        fn returns_empty_error_when_no_channels() {
            let channels: Vec<(String, usize)> = vec![];

            let result = FixtureMode::new(channels.into_iter());

            assert!(matches!(result, Err(FixtureModeCreateError::Empty)));
        }

        #[test]
        fn returns_duplicated_error_when_channel_name_duplicated() {
            let channels = vec![("Dimmer".to_string(), 0), ("Dimmer".to_string(), 1)];

            let result = FixtureMode::new(channels.into_iter());

            match result {
                Err(FixtureModeCreateError::Duplicated { duplicates }) => {
                    assert_eq!(duplicates.len(), 1);
                    match &duplicates[0] {
                        DuplicatedError::ChannelDuplicated { channel, offsets } => {
                            assert_eq!(channel, "Dimmer");
                            assert_eq!(offsets, &vec![0, 1]);
                        }
                        _ => panic!("Expected ChannelDuplicated error"),
                    }
                }
                _ => panic!("Expected Duplicated error"),
            }
        }

        #[test]
        fn returns_duplicated_error_when_offset_duplicated() {
            let channels = vec![("Dimmer".to_string(), 0), ("Red".to_string(), 0)];

            let result = FixtureMode::new(channels.into_iter());

            match result {
                Err(FixtureModeCreateError::Duplicated { duplicates }) => {
                    assert_eq!(duplicates.len(), 1);
                    match &duplicates[0] {
                        DuplicatedError::OffsetDuplicated { offset, channels } => {
                            assert_eq!(*offset, 0);
                            assert_eq!(channels, &vec!["Dimmer".to_string(), "Red".to_string()]);
                        }
                        _ => panic!("Expected OffsetDuplicated error"),
                    }
                }
                _ => panic!("Expected Duplicated error"),
            }
        }

        #[test]
        fn returns_not_contiguous_error_when_offsets_have_gap() {
            let channels = vec![
                ("Dimmer".to_string(), 0),
                ("Red".to_string(), 2), // gap: offset 1 is missing
            ];

            let result = FixtureMode::new(channels.into_iter());

            assert!(matches!(result, Err(FixtureModeCreateError::NotContiguous)));
        }

        #[test]
        fn returns_not_contiguous_error_when_offset_does_not_start_from_zero() {
            let channels = vec![("Dimmer".to_string(), 1), ("Red".to_string(), 2)];

            let result = FixtureMode::new(channels.into_iter());

            assert!(matches!(result, Err(FixtureModeCreateError::NotContiguous)));
        }

        #[test]
        fn returns_none_for_unknown_channel() {
            let channels = vec![("Dimmer".to_string(), 0)];
            let mode = FixtureMode::new(channels.into_iter()).unwrap();

            assert_eq!(mode.get_offset_by_channel("Unknown"), None);
        }

        #[test]
        fn returns_none_for_unknown_offset() {
            let channels = vec![("Dimmer".to_string(), 0)];
            let mode = FixtureMode::new(channels.into_iter()).unwrap();

            assert_eq!(mode.get_channel_by_offset(999), None);
        }

        #[test]
        fn collects_multiple_duplicated_errors() {
            let channels = vec![
                ("Dimmer".to_string(), 0),
                ("Dimmer".to_string(), 1), // channel duplicate
                ("Red".to_string(), 0),    // offset duplicate
            ];

            let result = FixtureMode::new(channels.into_iter());

            match result {
                Err(FixtureModeCreateError::Duplicated { duplicates }) => {
                    assert_eq!(duplicates.len(), 2);
                }
                _ => panic!("Expected Duplicated error"),
            }
        }
    }
}
