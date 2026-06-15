use std::{cell::{Ref, RefCell}, io::{BufRead, BufReader, Read}, rc::Rc};

use crate::{DictEntry, Error};

fn parse_key_values(buff: &str, line_num: usize) -> Result<DictEntry, Error> {
    if let Some((key, values_buff)) = buff.split_once('\t') {
        let values: Vec<String> = values_buff.split(' ')
            .map(|str| String::from(str))
            .collect();
        if values.is_empty() {
            Err(Error::InvalidTextDictinary("No value in an item".to_string(), line_num))
        } else if values.len() == 1 {
            Ok(DictEntry::new_with_key_and_value(key, &values[0]))
        } else {
            Ok(DictEntry::new_with_key_and_values(key, values))
        }
    } else {
        Err(Error::InvalidTextDictinary(format!("Tabular not found {}", buff), line_num))
    }
}

pub struct Lexicon {
    entries: RefCell<Vec<DictEntry>>
}

impl Lexicon {
    pub fn new() -> Self {
        Self { entries: RefCell::new(Vec::new()) }
    }

    pub fn from_entries(entries: Vec<DictEntry>) -> Self {
        Self { entries: RefCell::new(entries) }
    }

    pub fn add(&self, entry: DictEntry) {
        self.entries.borrow_mut().push(entry);
    }

    pub fn sort(&self) {
        self.entries.borrow_mut().sort();
    }

    pub fn is_sorted(&self) -> bool {
        self.entries.borrow().is_sorted_by(|a, b| a.key() < b.key())
    }

    pub fn is_unique(&self, dupkey: Option<&mut String>) -> bool {
        let entries = self.entries.borrow();
        for i in 1..entries.len() - 1 {
            if entries[i - 1].key() == entries[i].key() {
                if let Some(dupkey) = dupkey {
                    *dupkey = entries[i].key();
                }
                return false;
            }
        }
        true
    }

    pub fn get(&self, index: usize) -> Ref<'_, DictEntry> {
        Ref::map(self.entries.borrow(), |vec| &vec[index])
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn iter(&self) -> LexiconIter<'_> {
        self.into_iter()
    }

    pub fn partition_point<P>(&self, pred: P)-> usize
    where
        P: FnMut(&DictEntry) -> bool,
    {
        self.entries.borrow().partition_point(pred)
    }

    pub fn parse_lexicon_from<R: Read>(reader: R) -> Result<Rc<Lexicon>, Error> {
        let mut entries = Vec::new();
        let reader = BufReader::new(reader);
        for (i, line) in reader.lines().enumerate() {
            let line_num = i + 1;
            match parse_key_values(&line?, line_num) {
                Ok(entry) => entries.push(entry),
                Err(e) => return Err(e)
            }
        }
        Ok(Rc::new(Lexicon::from_entries(entries)))
    }
}

pub struct LexiconIter<'a> {
    borrow: Ref<'a, Vec<DictEntry>>,
    index: usize
}

impl<'a> LexiconIter<'a> {
    fn new(borrow: Ref<'a, Vec<DictEntry>>) -> Self {
        Self { borrow, index: 0 }
    }
}

impl<'a> Iterator for LexiconIter<'a> {
    type Item = Ref<'a, DictEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.borrow.len() {
            let e = Ref::map(Ref::clone(&self.borrow), |vec| &vec[self.index]);
            self.index += 1;
            Some(e)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.borrow.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> IntoIterator for &'a Lexicon {
    type Item = Ref<'a, DictEntry>;
    type IntoIter = LexiconIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexiconIter::new(self.entries.borrow())
    }
}

impl FromIterator<DictEntry> for Lexicon {
    fn from_iter<T: IntoIterator<Item = DictEntry>>(iter: T) -> Self {
        Self { entries: RefCell::new(Vec::from_iter(iter)) }
    }
}
