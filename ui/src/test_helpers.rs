use tsukuyomi_core::prelude::{
    Capability, CapabilityInner, ChannelDef, FixtureDef, FixtureMode, MergeMode,
};

///
pub fn make_fixture_def() -> FixtureDef {
    // TODO: tsukuyomi-coreに全く同じものがある
    let mut def = FixtureDef::new("Test Manufacturer", "Test Model");
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(
            MergeMode::HTP,
            Capability::Single(CapabilityInner::Intensity),
        ),
    );
    def.insert_channel(
        "Red",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Red)),
    );
    def.insert_channel(
        "Green",
        ChannelDef::new(
            MergeMode::HTP,
            tsukuyomi_core::fixture_def::Capability::Single(CapabilityInner::Green),
        ),
    );
    def.insert_channel(
        "Blue",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Blue)),
    );
    def.insert_mode(
        "4 Channel",
        FixtureMode::new(
            vec![
                ("Dimmer".into(), 0),
                ("Red".into(), 1),
                ("Green".into(), 2),
                ("Blue".into(), 3),
            ]
            .into_iter(),
        )
        .unwrap(),
    );
    def
}

///
pub fn make_fixture_def_2() -> FixtureDef {
    let mut def = FixtureDef::new("Some Manufacturer", "Some Model");
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(
            MergeMode::HTP,
            Capability::Single(CapabilityInner::Intensity),
        ),
    );
    def.insert_channel(
        "Red",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Red)),
    );
    def.insert_channel(
        "Green",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Green)),
    );
    def.insert_channel(
        "Blue",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::Blue)),
    );
    def.insert_channel(
        "White",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityInner::White)),
    );
    def.insert_mode(
        "5 Channel",
        FixtureMode::new(
            vec![
                ("Dimmer".into(), 0),
                ("Red".into(), 1),
                ("Green".into(), 2),
                ("Blue".into(), 3),
                ("White".into(), 4),
            ]
            .into_iter(),
        )
        .unwrap(),
    );
    def
}
