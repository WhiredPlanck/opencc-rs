use std::{borrow::Cow, sync::Arc};

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

    /// Split the text into segments. Unmatched characters are borrowed from
    /// `text` directly; only characters that matched a dictionary key are
    /// allocated as owned segments.
    pub fn segment<'a>(&self, text: &'a str) -> Vec<Cow<'a, str>> {
        match self {
            Segmentation::MaxMatch { dict: _, prefix_match } => text
                .char_indices()
                .map(|(i, ch)| {
                    let word = &text[i..i + ch.len_utf8()];
                    match prefix_match.match_prefix(word) {
                        Some(matched) => Cow::Owned(matched.key().to_string()),
                        None => Cow::Borrowed(word),
                    }
                })
                .collect(),
        }
    }
}
