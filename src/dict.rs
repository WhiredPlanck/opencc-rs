use std::{cmp::min, fs::File, path::Path, sync::Arc};

use enum_dispatch::enum_dispatch;

use crate::{DictEntry, DictGroup, Error, Lexicon, MarisaDict, SerializedValues, TextDict};

pub mod group;
pub mod text;
pub mod marisa;

/// Defines how a dictionary group resolves matches across child dictionaries.
pub enum DictGroupMatchPolicy {
    /// Preserve legacy DictGroup behavior: query child dictionaries in order and
    /// return the first dictionary that has a match. For prefix lookup, this means
    /// a shorter prefix from an earlier dictionary can win over a longer prefix
    /// from a later dictionary.
    ShortCircuit,
    /// Treat child dictionaries as a union for prefix lookup: the longest prefix
    /// across all children wins, with dictionary order breaking ties.
    Union
}

#[derive(Default)]
pub struct PrefixMatchResult {
    key: String,
    value: String,
}

impl PrefixMatchResult {
    pub fn new(key: &str, value: &str) -> Self {
        Self { key: key.to_string(), value: value.to_string() }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[enum_dispatch(Dict)]
pub enum AnyDict {
    Group(DictGroup),
    Marisa(MarisaDict),
    Text(TextDict),
    Serialized(SerializedValues)
}

#[enum_dispatch]
pub trait Dict: Send + Sync {
    fn key_max_length(&self) -> usize;

    fn lexicon(&self) -> Arc<Lexicon>;

    fn dict_group_items(&self) -> Option<&Vec<Arc<AnyDict>>> {
        None
    }

    fn match_policy(&self) -> DictGroupMatchPolicy {
        DictGroupMatchPolicy::ShortCircuit
    }

    fn supports_fast_prefix_match(&self) -> bool {
        false
    }

    fn identity(&self) -> usize;

    fn match_word(&self, _word: &str) -> Option<&DictEntry> {
        None
    }

    fn match_prefix(&self, word: &str) -> Option<&DictEntry> {
        let len = min(self.key_max_length(), word.len());
        word.char_indices()
            .take(len)
            .find_map(|(i, _)| self.match_word(&word[i..]))
    }

    fn match_all_prefix(&self, word: &str) -> Vec<&DictEntry> {
        let len = min(self.key_max_length(), word.len());
        word.char_indices()
            .take(len)
            .filter_map(|(i, _)| self.match_word(&word[i..]))
            .collect()
    }

    fn match_prefix_value(&self, word: &str) -> Option<PrefixMatchResult> {
        None
    }
}

pub trait SerializableDict {
    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error>;

    fn serialize_to_path(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        self.serialize_to_file(&mut file)
    }

    fn new_from_file(file: &mut File) -> Result<AnyDict, Error> where Self: Sized;

    fn new_from_path(path: &Path) -> Result<AnyDict, Error> where Self: Sized {
        let mut file = File::open(path)?;
        Self::new_from_file(&mut file)
    }
}
