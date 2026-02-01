use std::{
    collections::HashMap,
    fs::{self, File, read_dir},
    io,
    path::PathBuf,
};

use thiserror::Error;

use crate::prelude::{FixtureDef, FixtureDefId};

pub(super) trait FixtureDefRegistry {
    fn contains_def(&self, id: &FixtureDefId) -> bool;
    fn load(&mut self) -> Result<(), io::Error>;
}

/// ファイルからFixtureDefをロードしてキャッシュする
pub(super) struct FixtureDefRegistryImpl {
    path: PathBuf,
    defs: HashMap<FixtureDefId, FixtureDefCatalogItem>,
}

struct FixtureDefCatalogItem {
    manufacturer: String,
    model: String,
    path: PathBuf,
    val: Option<FixtureDef>,
}

impl FixtureDefRegistryImpl {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            defs: HashMap::new(),
        }
    }
    pub fn get_def(&self, id: &FixtureDefId) -> Result<&FixtureDef, FixtureDefGetError> {
        let item = self
            .defs
            .get(id)
            .ok_or(FixtureDefGetError::FixtureDefNotFound(*id))?;
        if let Some(val) = item.val {
            return Ok(&val);
        };

        let file = File::open(item.path)?;
    }
}

impl FixtureDefRegistry for FixtureDefRegistryImpl {
    fn contains_def(&self, id: &FixtureDefId) -> bool {
        self.defs.contains_key(id)
    }

    fn load(&mut self) -> Result<(), io::Error> {
        for ele in fs::read_dir(&self.path)? {}
        Ok(())
    }
}

pub enum FixtureDefLoadError {
    Def,
}

#[derive(Debug, Error)]
pub enum FixtureDefGetError {
    #[error(
        "fixture def {0:?} not found. You can reload catalogs by calling FixtureDefRegitry::load()."
    )]
    FixtureDefNotFound(FixtureDefId),
    #[error(transparent)]
    FileNotFound(#[from] io::Error),
}
