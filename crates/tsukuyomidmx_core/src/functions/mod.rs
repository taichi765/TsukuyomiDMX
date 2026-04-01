use crate::fixture::FixtureId;
use crate::functions::chaser::ChaserRuntime;
use crate::functions::static_scene::StaticSceneRuntime;
use std::ops::Deref;
use std::time::Duration;

mod chaser;
mod collection;
mod fader;
mod static_scene;
mod timeline;

pub use chaser::ChaserData;
pub use collection::Collection;
pub(crate) use fader::Fader;
pub use static_scene::SceneValue;
pub use static_scene::StaticSceneData;
use uuid::Uuid;

declare_id_newtype!(FunctionId);

pub trait FunctionDataGetters {
    fn id(&self) -> FunctionId;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub enum FunctionData {
    StaticScene(StaticSceneData),
    Chaser(ChaserData),
}

impl Deref for FunctionData {
    type Target = dyn FunctionDataGetters;

    fn deref(&self) -> &Self::Target {
        match self {
            FunctionData::StaticScene(data) => data,
            FunctionData::Chaser(data) => data,
        }
    }
}

impl FunctionData {
    pub(crate) fn create_runtime(&self) -> Box<dyn FunctionRuntime> {
        match self {
            FunctionData::StaticScene(_) => Box::new(StaticSceneRuntime::new()),
            FunctionData::Chaser(_) => Box::new(ChaserRuntime::new()),
        }
    }
}

pub trait FunctionRuntime: Send {
    fn run(&mut self, data: &FunctionData, tick_duration: Duration) -> Vec<FunctionCommand>;
}

/// [`FunctionRuntime::run()`] returns this and [`Engine`][crate::engine::Engine] evaluates the command
pub enum FunctionCommand {
    /// if the function is already started, `Engine` do nothing.
    StartFunction(FunctionId),
    /// if the function is already stoped, `Engine` do nothing.
    StopFuntion(FunctionId),
    WriteUniverse {
        fixture_id: FixtureId,
        channel: String,
        value: u8,
    },
    StartFade {
        from_id: FunctionId,
        to_id: FunctionId,
        chaser_id: FunctionId,
        duration: Duration,
    },
}

// helper functions for test
/*impl FunctionCommand {
    ///テスト用
    pub fn is_start_function(&self) -> bool {
        if let FunctionCommand::StartFunction(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_start_function_and(&self, want: usize) -> bool {
        if let FunctionCommand::StartFunction(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function(&self) -> bool {
        if let FunctionCommand::StopFuntion(_) = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_stop_function_and(&self, want: usize) -> bool {
        if let FunctionCommand::StopFuntion(have) = self
            && want == *have
        {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe(&self) -> bool {
        if let FunctionCommand::WriteUniverse { .. } = self {
            return true;
        }
        false
    }
    ///テスト用
    pub fn is_write_universe_and(&self, want: (u16, u8)) -> bool {
        if let FunctionCommand::WriteUniverse { address, value } = self
            && DmxAddress::new(want.0).unwrap() == *address
            && want.1 == *value
        {
            return true;
        }
        false
    }
}*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionType {
    Scene,
    Chaser,
    Fader,
    Collection,
    TimeLine,
    Other,
}

#[derive(Clone, Copy)]
pub struct FunctionInfo {
    pub id: usize,
    pub function_type: FunctionType,
}
