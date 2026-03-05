use std::{
    cell::{Ref, RefCell},
    cmp::max,
    fs::File,
    io::{Read, Write},
    rc::Rc,
};

use crate::{Dict, DictEntry, Error, Lexicon, SerializableDict};

pub struct TextDict {
    max_length: usize,
    lexicon: Rc<RefCell<Lexicon>>,
}

impl TextDict {
    pub fn from_lexicon(lexicon: Rc<RefCell<Lexicon>>) -> Self {
        let max_length = lexicon
            .borrow()
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
    fn lexicon(&self) -> Rc<RefCell<Lexicon>> {
        self.lexicon.clone()
    }

    fn key_max_length(&self) -> usize {
        self.max_length
    }

    fn match_word(&self, word: &str) -> Option<Ref<'_, dyn DictEntry>> {
        let guard = self.lexicon.borrow();
        let index = guard.partition_point(|x| x.key() < word);
        if index < guard.len() && guard[index].key() == word {
            Some(Ref::map(guard, |l| &l[index]))
        } else {
            None
        }
    }
}

impl SerializableDict for TextDict {
    fn new_from_file(file: &mut File) -> Result<Rc<TextDict>, Error> {
        match Lexicon::parse_lexicon_from(file) {
            Ok(lexicon) => {
                lexicon.borrow_mut().sort();
                if let Some(dupkey) = lexicon.borrow().dupkey() {
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
        let lexicon = self.lexicon.borrow();
        for entry in lexicon.iter() {
            writeln!(file, "{}", entry.to_string())?;
        }
        Ok(())
    }
}
