use std::{
    fs, io, path::{Path, PathBuf}, sync::{Arc, Weak}, time::UNIX_EPOCH
};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{
    AnyDict, Conversion, ConversionChain, Converter, DictGroup, Error, MarisaDict, Segmentation, SerializableDict, TextDict
};

static DICT_CACHE: Lazy<DashMap<String, Weak<AnyDict>>> =
    Lazy::new(|| DashMap::new());

fn prune_expired_dict_cache() {
    DICT_CACHE.retain(|_, dict| dict.upgrade().is_some());
}

fn get_file_cache_key(path: &Path, cache_prefix: &str) -> io::Result<String> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;


    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();
    let size = metadata.len();

    Ok(format!("{}\n{}\n{}.{:09}\n{}", cache_prefix, path.to_string_lossy(), secs, nanos, size))
}

#[derive(Deserialize, Debug, Clone)]
struct ConfigValue {
    name: Option<String>,
    segmentation: SegmentationKind,
    conversion_chain: Vec<ConversionValue>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
enum DictKind {
    Group(DictGroupValue),
    Text(DictValue),
    // Ocd(DictValue),
    Ocd2(DictValue),
}

#[derive(Deserialize, Debug, Clone)]
struct DictValue {
    pub file: String,
}

#[derive(Deserialize, Debug, Clone)]
struct DictGroupValue {
    dicts: Vec<DictKind>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
enum SegmentationKind {
    MMSeg(SegmentaionValue),
}

#[derive(Deserialize, Debug, Clone)]
struct SegmentaionValue {
    dict: DictKind,
}

#[derive(Deserialize, Debug, Clone)]
struct ConversionValue {
    dict: DictKind,
}

pub struct Config {
    paths: Vec<PathBuf>,
    argv0: Option<PathBuf>
}

impl Config {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            argv0: None
        }
    }

    pub fn paths(&mut self, paths: impl IntoIterator<Item = impl AsRef<Path>>) -> &mut Self {
        self.paths = paths
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        self
    }

    pub fn argv0(&mut self, argv0: Option<impl AsRef<Path>>) -> &mut Self {
        self.argv0 = argv0.map(|p| p.as_ref().to_path_buf());
        self
    }

    pub fn build(&mut self, path: impl AsRef<Path>) -> Result<Converter, Error> {
        if let Some(p) = &self.argv0 &&
            let Some(parent) = p.parent() {
                self.paths.push(PathBuf::from(parent));
        }
        
        let prefixed_file = self.find_config_file(path)?;
        if !prefixed_file.is_file() {
            let filename = prefixed_file.to_string_lossy().into_owned();
            return Err(Error::FileNotFound(filename));
        }
        let content = std::fs::read_to_string(prefixed_file)?;
        self.from_str(&content)
    }

    pub fn from_str(&self, json: &str) -> Result<Converter, Error> {
        let config: ConfigValue = serde_json::from_str(json)?;
        let name = config.name.unwrap_or_default();
        let segmentation = self.parse_segmentation(&config.segmentation)?;
        let conversion_chain = self.parse_conversion_chain(&config.conversion_chain)?;
        Ok(Converter::new(&name, segmentation, conversion_chain))
    }

    fn load_dict_with_paths<D: SerializableDict>(
        &self,
        cache_prefix: &str,
        filename: &str
    ) -> Result<Arc<AnyDict>, Error> {
        let mut candidates = vec![PathBuf::from(filename)];
        for dir_path in &self.paths {
            let path = dir_path.join(filename);
            candidates.push(path);
        }

        for path in &candidates {
            let cache_key = get_file_cache_key(path, cache_prefix);
            if cache_key.is_err() {
                continue;
            }
            let cache_key = cache_key.unwrap();
            {
                prune_expired_dict_cache();
                if let Some(cached) = DICT_CACHE.get(&cache_key) {
                    if let Some(dict) = cached.upgrade() {
                        return Ok(dict);
                    }
                }
            }

            if let Ok(dict) = D::new_from_path(filename.as_ref()) {
                prune_expired_dict_cache();
                {
                    if let Some(cached_dict) = DICT_CACHE.get(&cache_key)
                        .and_then(|entry| entry.upgrade()) {
                        return Ok(cached_dict);
                    }
                }
                DICT_CACHE.insert(cache_key, Arc::downgrade(&dict));
                return Ok(dict);
            }
        }
        Err(Error::FileNotFound(filename.to_string()))
    }

    fn parse_dict(&self, config: &DictKind) -> Result<Arc<AnyDict>, Error> {
        match config {
            DictKind::Group(group) => {
                let mut dicts = Vec::new();
                for kind in &group.dicts {
                    let dict = self.parse_dict(kind)?;
                    dicts.push(dict);
                }
                Ok(Arc::new(DictGroup::new(dicts).into()))
            }
            DictKind::Text(dict) => {
                let dict = self.load_dict_with_paths::<TextDict>("text", &dict.file)?;
                Ok(Arc::new(MarisaDict::from_dict(dict.as_ref()).into()))
            }
            DictKind::Ocd2(dict) => {
                let dict = self.load_dict_with_paths::<MarisaDict>("ocd2", &dict.file)?;
                Ok(dict)
            } // _ => unimplemented!(),
        }
    }

    fn parse_segmentation(&self, config: &SegmentationKind) -> Result<Segmentation, Error> {
        match config {
            SegmentationKind::MMSeg(segmentation) => {
                let dict = self.parse_dict(&segmentation.dict)?;
                Ok(Segmentation::new(dict))
            }
        }
    }

    fn parse_conversion_chain(
        &self,
        config: &[ConversionValue],
    ) -> Result<ConversionChain, Error> {
        let conversions = config
            .iter()
            .map(|conversion| {
                let dict = self.parse_dict(&conversion.dict)
                    .expect("Error on parsing dict");
                Conversion::new(dict)
            })
            .collect();
        Ok(ConversionChain::new(conversions))
    }

    fn find_config_file(&self, path: impl AsRef<Path>) -> Result<PathBuf, Error> {
        let path_ref = path.as_ref();
        // Working directory
        if path_ref.exists() {
            return Ok(path_ref.to_path_buf());
        }

        for dir_path in &self.paths {
            let path = dir_path.join(&path);
            if path.exists() {
                return Ok(path);
            }
        }
        Err(Error::FileNotFound(
            path.as_ref().to_string_lossy().into_owned(),
        ))
    }
}
