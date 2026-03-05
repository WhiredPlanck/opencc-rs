use std::{cell::{Ref, RefCell}, fs::File, path::Path, rc::Rc};

use crate::{DictEntry, Error, Lexicon};

pub mod group;
pub mod text;
pub mod marisa;

pub trait Dict {
    fn key_max_length(&self) -> usize;

    fn lexicon(&self) -> Rc<RefCell<Lexicon>>;

    fn match_word(&self, _word: &str) -> Option<Ref<'_, dyn DictEntry>> {
        None
    }

    fn match_prefix(&self, word: &str) -> Option<Ref<'_, dyn DictEntry>> {
        let max_len = self.key_max_length().min(word.len());
        let mut matched = None;
        for (i, c) in word.char_indices() {
            let prefix_end = i + c.len_utf8();
            if prefix_end > max_len {
                break;
            }
            if let Some(m) = self.match_word(&word[..prefix_end]) {
                matched = Some(m);
            }
        }
        matched
    }

    fn match_all_prefix(&self, word: &str) -> Vec<Ref<'_, dyn DictEntry>> {
        let max_len = self.key_max_length().min(word.len());
        let prefix_ends: Vec<usize> = word.char_indices()
            .map(|(i, c)| i + c.len_utf8())
            .take_while(|&end| end <= max_len)
            .collect();
        prefix_ends.into_iter().rev()
            .filter_map(|end| self.match_word(&word[..end]))
            .collect()
    }
}

pub trait SerializableDict {
    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error>;

    fn serialize_to_path(&self, path: &Path) -> Result<(), Error> {
        let mut file = File::create(path)?;
        self.serialize_to_file(&mut file)
    }

    fn new_from_file(file: &mut File) -> Result<Rc<Self>, Error> where Self: Sized;

    fn new_from_path(path: &Path) -> Result<Rc<Self>, Error> where Self: Sized {
        let mut file = File::open(path)?;
        Self::new_from_file(&mut file)
    }
}
