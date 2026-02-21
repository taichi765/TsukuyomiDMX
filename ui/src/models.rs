mod fixture;
mod fixture_def;

#[cfg(test)]
mod test_helpers {
    use std::{cell::RefCell, pin::Pin};

    use i_slint_core::model::ModelChangeListener;
    use tsukuyomi_core::prelude::{ChannelDef, ChannelKind, FixtureDef, FixtureMode, MergeMode};

    pub fn create_fixture_def() -> FixtureDef {
        let mut def = FixtureDef::new("Test Manufacturer", "Test Model");
        def.insert_channel(
            "Dimmer",
            ChannelDef::new(MergeMode::HTP, ChannelKind::Dimmer),
        );
        def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, ChannelKind::Red));
        def.insert_channel("Green", ChannelDef::new(MergeMode::HTP, ChannelKind::Green));
        def.insert_channel("Blue", ChannelDef::new(MergeMode::HTP, ChannelKind::Blue));
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

    pub fn create_fixture_def_2() -> FixtureDef {
        let mut def = FixtureDef::new("Some Manufacturer", "Some Model");
        def.insert_channel(
            "Dimmer",
            ChannelDef::new(MergeMode::HTP, ChannelKind::Dimmer),
        );
        def.insert_channel("Red", ChannelDef::new(MergeMode::HTP, ChannelKind::Red));
        def.insert_channel("Green", ChannelDef::new(MergeMode::HTP, ChannelKind::Green));
        def.insert_channel("Blue", ChannelDef::new(MergeMode::HTP, ChannelKind::Blue));
        def.insert_channel("White", ChannelDef::new(MergeMode::HTP, ChannelKind::White));
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

    pub struct SpyModelPeer {
        pub events: RefCell<Vec<DummyModelChangeEvent>>,
    }

    impl SpyModelPeer {
        pub fn new() -> Self {
            Self {
                events: RefCell::new(Vec::new()),
            }
        }
    }

    impl ModelChangeListener for SpyModelPeer {
        fn row_added(self: Pin<&Self>, index: usize, _count: usize) {
            self.events
                .borrow_mut()
                .push(DummyModelChangeEvent::Added(index));
        }

        fn row_changed(self: Pin<&Self>, row: usize) {
            self.events
                .borrow_mut()
                .push(DummyModelChangeEvent::Changed(row));
        }

        fn row_removed(self: Pin<&Self>, index: usize, _count: usize) {
            self.events
                .borrow_mut()
                .push(DummyModelChangeEvent::Removed(index));
        }

        fn reset(self: Pin<&Self>) {
            self.events.borrow_mut().push(DummyModelChangeEvent::Reset);
        }
    }

    pub enum DummyModelChangeEvent {
        Added(usize),
        Changed(usize),
        Removed(usize),
        Reset,
    }
}
