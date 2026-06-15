use std::{cell::Ref, cmp::max, collections::BTreeMap, rc::Rc};

use crate::{Dict, DictEntry, Lexicon};

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

    fn lexicon(&self) -> Rc<Lexicon> {
        let all_lexicon: Lexicon = self.dicts
            .iter()
            .flat_map(|dict| {
                dict.lexicon()
                    .iter()
                    .map(|entry| DictEntry::new_from_other(&entry))
                    .collect::<Vec<_>>()
            })
            .collect();
        all_lexicon.sort();
        Rc::new(all_lexicon)
    }

    fn match_word(&self, word: &str) -> Option<Ref<'_, DictEntry>> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_word(word))
    }

    fn match_prefix(&self, word: &str) -> Option<Ref<'_, DictEntry>> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_prefix(word))
    }

    fn match_all_prefix(&self, word: &str) -> Vec<Ref<'_, DictEntry>> {
        let mut matched: BTreeMap<usize, Ref<'_, DictEntry>> = BTreeMap::new();
        for dict in &self.dicts {
            let entries = dict.match_all_prefix(word);
            for entry in entries {
                let entry_len = entry.key().len();
                matched.entry(entry_len).or_insert(entry);
            }
        }
        matched.into_values().collect()
    }
}