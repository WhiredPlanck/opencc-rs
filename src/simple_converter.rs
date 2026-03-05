use std::path::Path;

use crate::{Config, Converter, Error};

pub struct SimpleConverter {
    converter: Converter
}

impl SimpleConverter {
    pub fn build<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::build_with_paths(path, Vec::new())
    }

    pub fn build_with_paths<P: AsRef<Path>>(path: P, paths: impl IntoIterator<Item = P>) -> Result<Self, Error> {
        Self::build_with_paths_and_argv0(path, paths, None)
    }
    
    pub fn build_with_paths_and_argv0<P: AsRef<Path>>(
        path: P, paths: impl IntoIterator<Item = P>, argv0: Option<P>
    ) -> Result<Self, Error> {
        let converter = Config::new()
            .paths(paths)
            .argv0(argv0)
            .build(path)?;
        Ok(Self { converter })
    }


    pub fn convert(&self, input: &str) -> String {
        self.converter.convert(input)
    }
}
