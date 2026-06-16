use std::sync::Arc;

use crate::{Dict, PrefixMatch};

pub enum Segmentation {
    MaxMatch { dict: Arc<dyn Dict>, prefix_match: PrefixMatch },
}

impl Segmentation {
    pub fn new(dict: Arc<dyn Dict>) -> Self {
        Self::MaxMatch { dict: dict.clone(), prefix_match: PrefixMatch::from_dict(&dict) }
    }

    pub fn dict(&self) -> Arc<dyn Dict> {
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
                        Some(matched) => matched.key,
                        None => word,
                    }
                })
                .collect(),
        }
    }
}
