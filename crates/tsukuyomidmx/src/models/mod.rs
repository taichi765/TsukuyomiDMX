mod fixture;
pub use fixture::*;
mod fixture_def;
pub use fixture_def::*;
mod manufacturer;
pub use manufacturer::*;
mod universe_view;
pub use universe_view::UniverseViewModel;

#[cfg(test)]
mod test_helpers {
    use std::{cell::RefCell, pin::Pin};

    use i_slint_core::model::ModelChangeListener;
    use tsukuyomidmx_core::prelude::{
        CapabilityKind, ChannelDef, FixtureDef, FixtureMode, MergeMode,
    };

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
