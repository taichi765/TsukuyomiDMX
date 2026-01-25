macro_rules! declare_id_newtype {
    ($(#[$outer:meta])* $name:ident) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl std::ops::Deref for $name {
            type Target = uuid::Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }


        impl From<uuid::Uuid> for $name {
            fn from(value: uuid::Uuid) -> Self {
                Self(value)
            }
        }

        impl Into<uuid::Uuid> for $name {
            fn into(self) -> uuid::Uuid {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_hyphenated())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::nil())
            }
        }
    };
}

mod readonly {
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, RwLock, RwLockReadGuard},
    };
    /// Read-only access to [`Arc<RwLock<T>>`].
    pub struct ArcView<T>(Arc<RwLock<T>>);

    impl<T> Clone for ReadOnly<T> {
        /// Cheap clone(same as [`Arc::clone()`]).
        ///
        /// `ReadOnly::clone(&self)` is recommended over `self.clone()` as same as [`Arc::clone()`].
        fn clone(&self) -> Self {
            ReadOnly(Arc::clone(&self.0))
        }
    }

    impl<T> ArcView<T> {
        pub fn new(value: Arc<RwLock<T>>) -> Self {
            Self(value)
        }

        pub fn read(&self) -> RwLockReadGuard<'_, T> {
            self.0.read().unwrap()
        }
    }

    /// Read-only facade of `Rc<RefCell<T>>`.
    ///
    /// `Arc` version of this is [`ArcView`].
    pub struct RcView<T>(Rc<RefCell<T>>);

    impl<T> RcView<T> {
        pub fn new(value: Rc<RefCell<T>>) -> Self {
            Self(value)
        }

        pub fn borrow(&self) -> std::cell::Ref<'_, T> {
            self.0.borrow()
        }
    }

    impl<T> Clone for RcView<T> {
        /// Cheap clone(same as [`Rc::clone()`]).
        fn clone(&self) -> Self {
            RcView(Rc::clone(&self.0))
        }
    }
}

pub use readonly::{ArcView, RcView};

pub mod engine;
pub mod fixture;
pub mod functions;
pub mod plugins;
//pub mod qxw_loader;
pub mod command_manager;
pub mod commands;
pub mod doc;
pub mod fixture_def;
pub mod universe;

pub mod prelude {
    pub use super::{
        doc::{DocStore, OutputPluginId},
        fixture::{Fixture, FixtureId, MergeMode},
        fixture_def::{ChannelDef, ChannelKind, FixtureDef, FixtureDefId, FixtureMode},
        functions::FunctionId,
        universe::{DmxAddress, UniverseId},
    };
}
