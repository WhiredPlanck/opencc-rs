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
        text.chars()
            .map(|pstr| {
                let word = pstr.to_string();
                match self.dict.match_prefix(&word) {
                    Some(matched) => matched.key(),
                    None => word
                }
            })
            .collect()
    }
}