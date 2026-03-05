use std::rc::Rc;

use crate::Dict;

pub trait Segmentation {
    fn segment(&self, text: &str) -> Vec<String>;
}

pub struct MaxMatchSegmentaion {
    dict: Rc<dyn Dict>
}

impl MaxMatchSegmentaion {
    pub fn new(dict: Rc<dyn Dict>) -> Self {
        Self { dict }
    }

    pub fn dict(&self) -> Rc<dyn Dict> {
        self.dict.clone()
    }
}

impl Segmentation for MaxMatchSegmentaion {
    fn segment(&self, text: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut start = 0;
        while start < text.len() {
            let suffix = &text[start..];
            match self.dict.match_prefix(suffix) {
                Some(matched) => {
                    let match_len = matched.key().len();
                    result.push(text[start..start + match_len].to_owned());
                    start += match_len;
                }
                None => {
                    let c_len = suffix.chars().next().unwrap().len_utf8();
                    result.push(suffix[..c_len].to_owned());
                    start += c_len;
                }
            }
        }
        result
    }
}