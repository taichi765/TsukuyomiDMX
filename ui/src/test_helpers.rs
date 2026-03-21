use tsukuyomi_core::prelude::{Capability, ChannelDef, FixtureDef, FixtureMode, MergeMode};

///
pub fn make_fixture_def() -> FixtureDef {
    // TODO: tsukuyomi-coreに全く同じものがある
    let mut def = FixtureDef::new("Test Manufacturer", "Test Model");
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(MergeMode::HTP, Capability::Intensity),
    );
    def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, Capability::Red));
    def.insert_channel("Green", ChannelDef::new(MergeMode::HTP, Capability::Green));
    def.insert_channel("Blue", ChannelDef::new(MergeMode::HTP, Capability::Blue));
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
        ChannelDef::new(MergeMode::HTP, Capability::Intensity),
    );
    def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, Capability::Red));
    def.insert_channel("Green", ChannelDef::new(MergeMode::HTP, Capability::Green));
    def.insert_channel("Blue", ChannelDef::new(MergeMode::HTP, Capability::Blue));
    def.insert_channel("White", ChannelDef::new(MergeMode::HTP, Capability::White));
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
