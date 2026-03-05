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
        let mut result = String::with_capacity(phrase.len());
        let mut start = 0;
        while start < phrase.len() {
            let suffix = &phrase[start..];
            match self.dict.match_prefix(suffix) {
                Some(matched) => {
                    let match_len = matched.key().len();
                    result.push_str(matched.get_default());
                    start += match_len;
                }
                None => {
                    let c = suffix.chars().next().unwrap();
                    result.push(c);
                    start += c.len_utf8();
                }
            }
        }
        result
    }

    pub fn convert_segments(&self, input: &[String]) -> Vec<String> {
        input.iter()
            .map(|segment| {
                self.convert_phrase(segment)
            })
            .collect()
    }
}