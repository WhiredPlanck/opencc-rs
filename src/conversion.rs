use std::{borrow::Cow, sync::Arc};

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

    /// Convert a phrase, character by character.
    ///
    /// Returns `Cow::Borrowed(phrase)` when nothing matched, so unchanged input
    /// costs no allocation. Only when at least one character is converted is an
    /// owned buffer allocated. A stack buffer replaces the per-character
    /// `to_string()` the previous implementation allocated.
    pub fn convert_phrase<'a>(&self, phrase: &'a str) -> Cow<'a, str> {
        let mut buf = [0u8; 4];
        let mut output: Option<String> = None;
        for ch in phrase.chars() {
            let word = ch.encode_utf8(&mut buf);
            match self.prefix_match.match_prefix(word) {
                Some(matched) => {
                    output
                        .get_or_insert_with(|| String::with_capacity(phrase.len()))
                        .push_str(matched.value());
                }
                None => {
                    if let Some(output) = output.as_mut() {
                        output.push(ch);
                    }
                }
            }
        }
        match output {
            Some(output) => Cow::Owned(output),
            None => Cow::Borrowed(phrase),
        }
    }

    /// Convert a single segment, consuming it so a `Cow::Owned` segment that
    /// stays unchanged is reused instead of cloned.
    pub fn convert_segment<'a>(&self, segment: Cow<'a, str>) -> Cow<'a, str> {
        match segment {
            Cow::Borrowed(phrase) => self.convert_phrase(phrase),
            Cow::Owned(phrase) => match self.convert_phrase(&phrase) {
                // Unchanged: keep the owned string we already had.
                Cow::Borrowed(_) => Cow::Owned(phrase),
                Cow::Owned(converted) => Cow::Owned(converted),
            },
        }
    }

    pub fn convert_segments<'a>(&self, input: &'a [Cow<'a, str>]) -> Vec<Cow<'a, str>> {
        input
            .iter()
            .map(|segment| self.convert_segment(segment.clone()))
            .collect()
    }
}
