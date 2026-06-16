use std::{
    cell::Ref, cmp::max, fs::File, io::{Read, Write}, rc::Rc
};

use crate::{Dict, DictEntry, Error, Lexicon, SerializableDict};

pub struct TextDict {
    max_length: usize,
    lexicon: Rc<Lexicon>,
}

impl TextDict {
    pub fn from_lexicon(lexicon: Rc<Lexicon>) -> Self {
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

    pub fn from_sorted<R: Read>(reader: R) -> Result<Rc<TextDict>, Error> {
        match Lexicon::parse_lexicon_from(reader) {
            Ok(lexicon) => Ok(Rc::new(TextDict::from_lexicon(lexicon))),
            Err(e) => Err(e),
        }
    }

    pub fn from_dict(dict: &dyn Dict) -> Rc<Self> {
        Rc::new(TextDict::from_lexicon(dict.lexicon()))
    }
}

impl Dict for TextDict {
    fn lexicon(&self) -> Rc<Lexicon> {
        self.lexicon.clone()
    }

    fn key_max_length(&self) -> usize {
        self.max_length
    }

    fn identity(&self) -> usize {
        self as *const Self as usize
    }

    fn match_word(&self, word: &str) -> Option<Ref<'_, DictEntry>> {
        let entry= DictEntry::NoValue { key: word.to_string() };
        let lexicon = &self.lexicon;
        let index = lexicon.partition_point(|x| x < &entry);
        if index < lexicon.len() {
            Some(lexicon.get(index))
        } else {
            None
        }
    }
}

impl SerializableDict for TextDict {
    fn new_from_file(file: &mut File) -> Result<Rc<dyn Dict>, Error> {
        match Lexicon::parse_lexicon_from(file) {
            Ok(lexicon) => {
                lexicon.sort();
                let mut dupkey = String::new();
                if lexicon.is_unique(Some(&mut dupkey)) {
                    return Err(Error::InvalidFormat(format!(
                        "The text dictionary contains duplicated keys: {}.",
                        dupkey
                    )));
                }
                Ok(Rc::new(TextDict::from_lexicon(lexicon)))
            }
            Err(e) => Err(e),
        }
    }

    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error> {
        let lexicon = &self.lexicon;
        for entry in lexicon.iter() {
            writeln!(file, "{}", entry.to_string())?;
        }
        Ok(())
    }
}
