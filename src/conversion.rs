use std::rc::Rc;

use crate::Dict;

pub struct Conversion {
    dict: Rc<dyn Dict>
}

impl Conversion {
    pub fn new(dict: Rc<dyn Dict>) -> Self {
        Self { dict }
    }

    pub fn dict(&self) -> Rc<dyn Dict> {
        self.dict.clone()
    }

    pub fn convert_phrase(&self, phrase: &str) -> String {
        // let mut result = String::with_capacity(phrase.len());
        // let mut last_end = 0;

        // for (i, c) in phrase.char_indices() {
        //     let mut buffer = [0u8; 4];
        //     let s = c.encode_utf8(&mut buffer);
        //     if let Some(matched) = self.dict.match_prefix(s) {
        //         if i > last_end {
        //             result.push_str(&phrase[last_end..i]);
        //         }
        //         result.push_str(&matched.get_default());
        //         last_end = i + c.len_utf8();
        //     }
        // }
        // if last_end < phrase.len() {
        //     result.push_str(&phrase[last_end..]);
        // }
        // result
        phrase.chars()
            .map(|pstr| {
                let word = pstr.to_string();
                match self.dict.match_prefix(&word) {
                    Some(matched) => matched.get_default(),
                    None => word
                }
            })
            .collect()
    }

    pub fn convert_segments(&self, input: &Vec<String>) -> Vec<String> {
        input.iter()
            .map(|segment| {
                self.convert_phrase(segment)
            })
            .collect()
    }
}