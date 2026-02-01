use std::{
    cell::OnceCell,
    collections::HashMap,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    fixture_def::FixtureDefConverseError,
    prelude::{FixtureDef, FixtureDefId},
};

pub(super) trait FixtureDefRegistry {
    fn contains(&self, id: &FixtureDefId) -> bool;
    fn load(&mut self) -> Result<(), io::Error>;
    fn get<'a>(&'a self, id: &FixtureDefId) -> Result<&'a FixtureDef, FixtureDefLookupError<'a>>;
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
    val: OnceCell<Result<FixtureDef, FixtureDefLoadError>>,
}

impl FixtureDefRegistryImpl {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            defs: HashMap::new(),
        }
    }
}

impl FixtureDefRegistry for FixtureDefRegistryImpl {
    fn contains(&self, id: &FixtureDefId) -> bool {
        self.defs.contains_key(id)
    }

    fn get<'a>(&'a self, id: &FixtureDefId) -> Result<&'a FixtureDef, FixtureDefLookupError<'a>> {
        let item = self
            .defs
            .get(id)
            .ok_or(FixtureDefLookupError::NotInCatalog(*id))?;

        item.val
            .get_or_init(|| {
                let s = fs::read_to_string(&item.path)?;
                let dto: ofl_schemas::Fixture = serde_json::from_str(&s)?;
                FixtureDef::try_from(dto)
                    .map_err(|e| FixtureDefLoadError::FixtureDefConverseFailed(e))
            })
            .as_ref()
            .map_err(|e| FixtureDefLookupError::LoadFailed(e).to_owned())
    }

    fn load(&mut self) -> Result<(), io::Error> {
        let new = self
            .path
            .read_dir()?
            .filter_map(Result::ok)
            .filter(|ent| ent.path().is_dir())
            .fold(HashMap::new(), |mut map, dir| {
                let manufacturer = dir.file_name().into_string().expect("todo");
                fs::read_dir(dir.path())
                    .expect("todo: 無視しつつdiagnostics出す")
                    .filter_map(Result::ok)
                    .filter(|ent| ent.path().is_file())
                    .for_each(|file| {
                        let model = file.file_name().into_string().expect("todo");
                        map.insert(
                            FixtureDefId::new(),
                            FixtureDefCatalogItem {
                                manufacturer: manufacturer.clone(), // TODO: Cow使った方がいいのでは？
                                model,
                                path: file.path(),
                                val: OnceCell::new(),
                            },
                        );
                    });
                map
            });

        self.defs.clear();
        self.defs = new;
        Ok(())
    }
}

#[derive(Debug, Error, Clone)]
pub enum FixtureDefLookupError<'a> {
    #[error(
        "fixture def {0:?} not found in catalog. You can reload catalogs by calling FixtureDefRegitry::load()."
    )]
    NotInCatalog(FixtureDefId),
    #[error("failed to load fixture def from file: {0:?}")]
    LoadFailed(&'a FixtureDefLoadError),
}

#[derive(Debug, Error)]
pub enum FixtureDefLoadError {
    #[error("fixture def recoded in catalog but file didn't exist: {0:?}")]
    FileNotFound(#[from] io::Error),
    #[error(transparent)]
    JsonParseFailed(#[from] serde_json::Error),
    #[error(transparent)]
    FixtureDefConverseFailed(#[from] FixtureDefConverseError),
}
