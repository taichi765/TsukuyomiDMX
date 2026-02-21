use std::{cell::OnceCell, collections::HashMap, fs, io, path::PathBuf, sync::Arc};

use thiserror::Error;

use crate::{
    fixture_def::FixtureDefConverseError,
    prelude::{FixtureDef, FixtureDefId},
};

pub trait FixtureDefRegistry {
    fn contains(&self, id: &FixtureDefId) -> bool;
    fn load(&mut self) -> Result<(), io::Error>;
    fn get<'a>(&'a self, id: &FixtureDefId) -> Result<&'a FixtureDef, FixtureDefLookupError>;
    fn iter_metadata<'a>(&'a self) -> Box<dyn Iterator<Item = FixtureDefMetaData<'a>> + 'a>;
}

/// ファイルからFixtureDefをロードしてキャッシュする
pub struct FixtureDefRegistryImpl {
    search_path: PathBuf, // TODO: Vecの方がよい？
    defs: HashMap<FixtureDefId, FixtureDefCatalogItem>,
}

struct FixtureDefCatalogItem {
    manufacturer: String,
    model: String,
    path: PathBuf,
    val: OnceCell<Result<FixtureDef, Arc<FixtureDefLoadError>>>,
}

impl FixtureDefRegistryImpl {
    pub fn new(path: PathBuf) -> Self {
        Self {
            search_path: path,
            defs: HashMap::new(),
        }
    }
}

impl FixtureDefRegistry for FixtureDefRegistryImpl {
    fn contains(&self, id: &FixtureDefId) -> bool {
        self.defs.contains_key(id)
    }

    fn get<'a>(&'a self, id: &FixtureDefId) -> Result<&'a FixtureDef, FixtureDefLookupError> {
        let item = self
            .defs
            .get(id)
            .ok_or(FixtureDefLookupError::NotInCatalog(id.clone()))?;

        item.val
            .get_or_init(|| {
                let s = fs::read_to_string(&item.path).map_err(|e| Arc::new(e.into()))?;
                let dto: ofl_schemas::Fixture =
                    serde_json::from_str(&s).map_err(|e| Arc::new(e.into()))?;
                FixtureDef::try_from(dto).map_err(|e| Arc::new(e.into()))
            })
            .as_ref()
            .map_err(|e| FixtureDefLookupError::LoadFailed(Arc::clone(e)))
    }

    fn load(&mut self) -> Result<(), io::Error> {
        let new = self
            .search_path
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
                        let model = file
                            .path()
                            .file_stem()
                            .expect("todo")
                            .to_str()
                            .expect("todo")
                            .to_string();
                        map.insert(
                            FixtureDefId::new(manufacturer.clone(), model.clone()),
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

    fn iter_metadata<'a>(&'a self) -> Box<dyn Iterator<Item = FixtureDefMetaData<'a>> + 'a> {
        Box::new(self.defs.iter().map(|(id, item)| FixtureDefMetaData {
            id,
            manufacturer: &item.manufacturer,
            model: &item.model,
        }))
    }
}

pub struct FixtureDefMetaData<'a> {
    pub id: &'a FixtureDefId,
    pub manufacturer: &'a str,
    pub model: &'a str,
}

#[derive(Debug, Error)]
pub enum FixtureDefLookupError {
    #[error(
        "fixture def {0:?} not found in catalog. You can reload catalogs by calling FixtureDefRegitry::load()."
    )]
    NotInCatalog(FixtureDefId),
    #[error("failed to load fixture def from file: {0:?}")]
    LoadFailed(Arc<FixtureDefLoadError>),
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
