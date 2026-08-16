use std::{
    cmp::max, fs::File, io::{Read, Write}, sync::Arc
};

use crate::{AnyDict, Dict, DictEntry, Error, Lexicon, SerializableDict};

pub struct TextDict {
    max_length: usize,
    lexicon: Arc<Lexicon>,
}

impl TextDict {
    pub fn from_lexicon(lexicon: Arc<Lexicon>) -> Self {
        let max_length = lexicon
            .iter()
            .fold(0, |acc, entry| max(acc, entry.key().len()));
        // assert!(lexicon.borrow().is_sorted());
        // assert!(lexicon.borrow().dupkey().is_none());
        Self {
            max_length,
            lexicon,
        }
    }

    pub fn from_sorted<R: Read>(reader: R) -> Result<Self, Error> {
        match Lexicon::parse_lexicon_from(reader) {
            Ok(lexicon) => Ok(TextDict::from_lexicon(Arc::new(lexicon))),
            Err(e) => Err(e),
        }
    }

    pub fn from_dict(dict: &AnyDict) -> Self {
        TextDict::from_lexicon(dict.lexicon())
    }
}

impl Dict for TextDict {
    fn lexicon(&self) -> Arc<Lexicon> {
        self.lexicon.clone()
    }

    fn key_max_length(&self) -> usize {
        self.max_length
    }

    fn identity(&self) -> usize {
        self as *const Self as usize
    }

    fn match_word(&self, word: &str) -> Option<&DictEntry> {
        let lexicon = &self.lexicon;
        // Binary search with a borrowed key instead of allocating a temporary
        // DictEntry for every lookup.
        let index = lexicon.partition_point(|x| x.key() < word);
        if index < lexicon.len() && lexicon.get(index).key() == word {
            Some(lexicon.get(index))
        } else {
            None
        }
    }
}

impl SerializableDict for TextDict {
    fn new_from_file(file: &mut File) -> Result<AnyDict, Error> {
        match Lexicon::parse_lexicon_from(file) {
            Ok(mut lexicon) => {
                lexicon.sort();
                let mut dupkey = String::new();
                if lexicon.is_unique(Some(&mut dupkey)) {
                    return Err(Error::InvalidFormat(format!(
                        "The text dictionary contains duplicated keys: {}.",
                        dupkey
                    )));
                }
                Ok(AnyDict::Text(TextDict::from_lexicon(Arc::new(lexicon))))
            }
            Err(e) => Err(e),
        }
    }

    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error> {
        let lexicon = &self.lexicon;
        for entry in lexicon.iter() {
            // `Display` writes directly into the file, avoiding the
            // intermediate String that `entry.to_string()` would allocate.
            writeln!(file, "{}", entry)?;
        }
        Ok(())
    }
}
