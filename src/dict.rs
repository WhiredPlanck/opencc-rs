use std::{cmp::min, fs::File, path::Path, sync::Arc};

use crate::{DictEntry, Error, Lexicon};

pub mod group;
pub mod text;
pub mod marisa;

pub trait Dict: Send + Sync {
    fn key_max_length(&self) -> usize;

    fn lexicon(&self) -> Arc<Lexicon>;

    fn dict_group_items(&self) -> Option<&Vec<Arc<dyn Dict>>> {
        None
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
}

pub trait SerializableDict {
    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error>;

    fn serialize_to_path(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        self.serialize_to_file(&mut file)
    }

    fn new_from_file(file: &mut File) -> Result<Arc<dyn Dict>, Error> where Self: Sized;

    fn new_from_path(path: &Path) -> Result<Arc<dyn Dict>, Error> where Self: Sized {
        let mut file = File::open(path)?;
        Self::new_from_file(&mut file)
    }
}
