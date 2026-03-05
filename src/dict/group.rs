use std::{cell::{Ref, RefCell}, cmp::max, collections::BTreeMap, rc::Rc};

use crate::{Dict, DictEntry, DictEntryFactory, Lexicon};

pub struct DictGroup {
    key_max_length: usize,
    dicts: Vec<Rc<dyn Dict>>
}

impl DictGroup {
    pub fn new(dicts: Vec<Rc<dyn Dict>>) -> Self {
        let key_max_length = dicts.iter()
            .fold(0, |acc, e| {
                max(acc, e.key_max_length())
            });
        Self { key_max_length, dicts }
    }

    pub fn dicts(&self) -> &Vec<Rc<dyn Dict>> {
        &self.dicts
    }
}

impl Dict for DictGroup {
    fn key_max_length(&self) -> usize {
        self.key_max_length
    }

    fn lexicon(&self) -> Rc<RefCell<Lexicon>> {
        let mut all_lexicon: Lexicon = self.dicts
            .iter()
            .flat_map(|dict| {
                let lexicon = dict.lexicon();
                let borrowed = lexicon.borrow();
                borrowed
                    .iter()
                    .map(|entry| DictEntryFactory::new_from_other(entry.as_ref()))
                    .collect::<Vec<_>>()
            })
            .collect();
        all_lexicon.sort();
        Rc::new(RefCell::new(all_lexicon))
    }

    fn match_word(&self, word: &str) -> Option<Ref<'_, dyn DictEntry>> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_word(word))
    }

    fn match_prefix(&self, word: &str) -> Option<Ref<'_, dyn DictEntry>> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_prefix(word))
    }

    fn match_all_prefix(&self, word: &str) -> Vec<Ref<'_, dyn DictEntry>> {
        let mut matched: BTreeMap<usize, Ref<'_, dyn DictEntry>> = BTreeMap::new();
        for dict in &self.dicts {
            let entries = dict.match_all_prefix(word);
            for entry in entries {
                let entry_len = entry.key().len();
                if !matched.contains_key(&entry_len) {
                    matched.insert(entry_len, entry);
                }
            }
        }
        matched.into_values().collect()
    }
}