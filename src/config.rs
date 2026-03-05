use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use serde::Deserialize;

use crate::{
    Conversion, ConversionChain, Converter, Dict, DictGroup, Error, MarisaDict, MaxMatchSegmentaion, Segmentation, SerializableDict, TextDict
};

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
    argv0: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            argv0: None,
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
        if let Some(p) = &self.argv0 {
            if let Some(parent) = p.parent() {
                self.paths.push(PathBuf::from(parent));
            }
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

    fn load_dict_with_paths<D: SerializableDict, P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Rc<D>, Error> {
        // Working directory
        let dict = D::new_from_path(path.as_ref());
        if dict.is_ok() {
            return dict;
        }
        for dir_path in &self.paths {
            let path = dir_path.join(&path);
            let dict = D::new_from_path(&path);
            if dict.is_ok() {
                return dict;
            }
        }
        let filename = path.as_ref().to_string_lossy().into_owned();
        Err(Error::FileNotFound(filename))
    }

    fn parse_dict(&self, config: &DictKind) -> Result<Rc<dyn Dict>, Error> {
        match config {
            DictKind::Group(group) => {
                let mut dicts = Vec::new();
                for kind in &group.dicts {
                    let dict = self.parse_dict(&kind)?;
                    dicts.push(dict);
                }
                Ok(Rc::new(DictGroup::new(dicts)))
            }
            DictKind::Text(dict) => {
                let dict: Rc<TextDict> = self.load_dict_with_paths(&dict.file)?;
                Ok(MarisaDict::from_dict(dict.as_ref()))
            }
            DictKind::Ocd2(dict) => {
                let dict: Rc<MarisaDict> = self.load_dict_with_paths(&dict.file)?;
                Ok(dict)
            } // _ => unimplemented!(),
        }
    }

    fn parse_segmentation(&self, config: &SegmentationKind) -> Result<Rc<dyn Segmentation>, Error> {
        match config {
            SegmentationKind::MMSeg(segmentation) => {
                let dict = self.parse_dict(&segmentation.dict)?;
                Ok(Rc::new(MaxMatchSegmentaion::new(dict)))
            }
        }
    }

    fn parse_conversion_chain(
        &self,
        config: &Vec<ConversionValue>,
    ) -> Result<Rc<ConversionChain>, Error> {
        let mut conversions = Vec::new();
        for conversion in config {
            let dict = self.parse_dict(&conversion.dict)?;
            conversions.push(Conversion::new(dict));
        }
        Ok(Rc::new(ConversionChain::new(conversions)))
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
