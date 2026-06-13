use std::rc::Rc;

use crate::Dict;

pub enum Segmentation {
    MaxMatch { dict: Rc<dyn Dict> },
}

impl Segmentation {
    pub fn new(dict: Rc<dyn Dict>) -> Self {
        Self::MaxMatch { dict }
    }

    pub fn dict(&self) -> Rc<dyn Dict> {
        match self {
            Segmentation::MaxMatch { dict } => dict.clone(),
        }
    }

    pub fn segment(&self, text: &str) -> Vec<String> {
        match self {
            Segmentation::MaxMatch { dict } => text
                .chars()
                .map(|pstr| {
                    let word = pstr.to_string();
                    match dict.match_prefix(&word) {
                        Some(matched) => matched.key(),
                        None => word,
                    }
                })
                .collect(),
        }
    }
}
