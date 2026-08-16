use std::sync::Arc;

use crate::{AnyDict, PrefixMatch};

pub enum Segmentation {
    MaxMatch { dict: Arc<AnyDict>, prefix_match: PrefixMatch },
}

impl Segmentation {
    pub fn new(dict: Arc<AnyDict>) -> Self {
        Self::MaxMatch { dict: dict.clone(), prefix_match: PrefixMatch::from_dict(&dict) }
    }

    pub fn dict(&self) -> Arc<AnyDict> {
        match self {
            Segmentation::MaxMatch { dict, .. } => dict.clone(),
        }
    }

    pub fn segment(&self, text: &str) -> Vec<String> {
        match self {
            Segmentation::MaxMatch { dict: _, prefix_match } => text
                .chars()
                .map(|pstr| {
                    let word = pstr.to_string();
                    match prefix_match.match_prefix(&word) {
                        Some(matched) => matched.key().to_string(),
                        None => word,
                    }
                })
                .collect(),
        }
    }
}
