use std::{cmp::max, collections::BTreeMap, sync::Arc};

use crate::{AnyDict, Dict, DictEntry, Lexicon};

pub struct DictGroup {
    key_max_length: usize,
    dicts: Vec<Arc<AnyDict>>
}

impl DictGroup {
    pub fn new(dicts: Vec<Arc<AnyDict>>) -> Self {
        let key_max_length = dicts.iter()
            .fold(0, |acc, e| {
                max(acc, e.key_max_length())
            });
        Self { key_max_length, dicts }
    }

    pub fn dicts(&self) -> &Vec<Arc<AnyDict>> {
        &self.dicts
    }
}

impl Dict for DictGroup {
    fn key_max_length(&self) -> usize {
        self.key_max_length
    }

    fn lexicon(&self) -> Arc<Lexicon> {
        let mut all_lexicon: Lexicon = self.dicts
            .iter()
            .flat_map(|dict| {
                dict.lexicon()
                    .iter()
                    .map(|entry| DictEntry::new_from_other(&entry))
                    .collect::<Vec<_>>()
            })
            .collect();
        all_lexicon.sort();
        Arc::new(all_lexicon)
    }

    fn dict_group_items(&self) -> Option<&Vec<Arc<AnyDict>>> {
        Some(&self.dicts)
    }

    fn identity(&self) -> usize {
        self as *const Self as usize
    }

    fn match_word(&self, word: &str) -> Option<&DictEntry> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_word(word))
    }

    fn match_prefix(&self, word: &str) -> Option<&DictEntry> {
        self.dicts
            .iter()
            .find_map(|dict| dict.match_prefix(word))
    }

    fn match_all_prefix(&self, word: &str) -> Vec<&DictEntry> {
        let mut matched: BTreeMap<usize, &DictEntry> = BTreeMap::new();
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