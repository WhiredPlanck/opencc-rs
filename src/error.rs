use std::{io, str::Utf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid text dictionary at line {1}: {0}")]
    InvalidTextDictinary(String, usize),

    #[error("Invalid UTF8")]
    InvalidUTF8(#[from] Utf8Error),

    #[error("Error on Input / Output")]
    InvalidIO(#[from] io::Error),

    #[error("{0} not found or not accessible")]
    FileNotFound(String)
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::InvalidFormat(value.to_string())
    }
}
