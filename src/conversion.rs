use std::sync::Arc;

use crate::{AnyDict, PrefixMatch};

pub struct Conversion {
    dict: Arc<AnyDict>,
    prefix_match: PrefixMatch
}

impl Conversion {
    pub fn new(dict: Arc<AnyDict>) -> Self {
        Self { dict: dict.clone(), prefix_match: PrefixMatch::from_dict(&dict) }
    }

    pub fn dict(&self) -> Arc<AnyDict> {
        self.dict.clone()
    }

    pub fn convert_phrase(&self, phrase: &str) -> String {
        phrase.chars()
            .map(|pstr| {
                let word = pstr.to_string();
                match self.prefix_match.match_prefix(&word) {
                    Some(matched) => matched.value().to_string(),
                    None => word
                }
            })
            .collect()
    }

    pub fn convert_segments(&self, input: &[String]) -> Vec<String> {
        input.iter()
            .map(|segment| {
                self.convert_phrase(segment)
            })
            .collect()
    }
}
