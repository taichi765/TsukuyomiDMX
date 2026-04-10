use tsukuyomidmx_core::prelude::{
    Capability, CapabilityKind, ChannelDef, FixtureDef, FixtureMode, MergeMode,
};

///
pub fn make_fixture_def_2() -> FixtureDef {
    let mut def = FixtureDef::new("Some Manufacturer", "Some Model");
    def.insert_channel(
        "Dimmer",
        ChannelDef::new(
            MergeMode::HTP,
            Capability::Single(CapabilityKind::Intensity),
        ),
    );
    def.insert_channel(
        "Red",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityKind::Red)),
    );
    def.insert_channel(
        "Green",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityKind::Green)),
    );
    def.insert_channel(
        "Blue",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityKind::Blue)),
    );
    def.insert_channel(
        "White",
        ChannelDef::new(MergeMode::HTP, Capability::Single(CapabilityKind::White)),
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
