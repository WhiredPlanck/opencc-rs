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

    pub fn convert_segments(&self, input: &[String]) -> Vec<String> {
        input.iter()
            .map(|segment| {
                self.convert_phrase(segment)
            })
            .collect()
    }
}