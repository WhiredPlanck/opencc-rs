use std::path::Path;

use crate::{Config, ConversionChain, Error, Segmentation};

pub struct Converter {
    _name: String,
    segmentation: Segmentation,
    conversion_chain: ConversionChain
}

impl Converter {
    pub fn new(
        name: &str,
        segmentation: Segmentation,
        conversion_chain: ConversionChain
    ) -> Self {
        Self { _name: name.to_string(), segmentation, conversion_chain }
    }

    pub fn build<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Config::new().build(path)
    }

    pub fn from_str_with_paths(json: &str, paths: impl IntoIterator<Item = impl AsRef<Path>>) -> Result<Self, Error> {
        Config::new().paths(paths).from_str(json)
    }

    pub fn from_str_with_dir<P: AsRef<Path>>(json: &str, config_dir: P) -> Result<Self, Error> {
        let paths = vec![config_dir];
        Self::from_str_with_paths(json, &paths)
    }

    pub fn segmentation(&self) -> &Segmentation {
        &self.segmentation
    }

    pub fn conversion_chain(&self) -> &ConversionChain {
        &self.conversion_chain
    }

    pub fn convert(&self, text: &str) -> String {
        let segments = self.segmentation.segment(text);
        let converted = self.conversion_chain.convert(&segments);
        let mut output = String::with_capacity(text.len());
        for segment in converted {
            output.push_str(&segment);
        }
        output
    }
}
