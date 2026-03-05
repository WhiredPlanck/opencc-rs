use std::{cell::RefCell, io::{BufRead, BufReader, Read}, ops::{Index, IndexMut}, rc::Rc};

use crate::{DictEntry, DictEntryFactory, Error};

fn parse_key_values(buff: &str, line_num: usize) -> Result<Box<dyn DictEntry>, Error> {
    if let Some((key, values_buff)) = buff.split_once('\t') {
        let values: Vec<String> = values_buff.split(' ')
            .map(|str| str.to_owned())
            .collect();
        if values.is_empty() {
            Err(Error::InvalidTextDictinary("No value in an item".to_string(), line_num))
        } else if values.len() == 1 {
            Ok(DictEntryFactory::new_with_key_and_value(key, &values[0]))
        } else {
            Ok(DictEntryFactory::new_with_key_and_values(key, values))
        }
    } else {
        Err(Error::InvalidTextDictinary(format!("Tabular not found {}", buff), line_num))
    }
}

pub struct Lexicon {
    entries: Vec<Box<dyn DictEntry>>
}

impl Lexicon {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn from_entries(entries: Vec<Box<dyn DictEntry>>) -> Self {
        Self { entries }
    }

    pub fn add(&mut self, entry: Box<dyn DictEntry>) {
        self.entries.push(entry);
    }

    pub fn sort(&mut self) {
        self.entries.sort();
    }

    pub fn is_sorted(&self) -> bool {
        self.entries.is_sorted_by(|a, b| a.key() < b.key())
    }

    pub fn dupkey(&self) -> Option<String> {
        for i in 1..self.entries.len() {
            if self.entries[i - 1].key() == self.entries[i].key() {
                return Some(self.entries[i].key().to_owned());
            }
        }
        None
    }

    pub fn get(&self, index: usize) -> &dyn DictEntry {
        self.entries[index].as_ref()
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Box<dyn DictEntry>> {
        self.entries.iter()
    }

    pub fn partition_point<P>(&self, pred: P)-> usize
    where
        P: FnMut(&Box<dyn DictEntry>) -> bool,
    {
        self.entries.partition_point(pred)
    }

    pub fn parse_lexicon_from<R: Read>(reader: R) -> Result<Rc<RefCell<Lexicon>>, Error> {
        let mut lexicon = Lexicon::from_entries(vec![]);
        let reader = BufReader::new(reader);
        for (i, line) in reader.lines().enumerate() {
            let line_num = i + 1;
            match parse_key_values(&line?, line_num) {
                Ok(entry) => lexicon.add(entry),
                Err(e) => return Err(e)
            }
        }
        Ok(Rc::new(RefCell::new(lexicon)))
    }
}

impl Index<usize> for Lexicon {
    type Output = dyn DictEntry;

    fn index(&self, index: usize) -> &Self::Output {
        self.entries[index].as_ref()
    }
}

impl IndexMut<usize> for Lexicon {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.entries[index].as_mut()
    }
}

impl IntoIterator for Lexicon {
    type Item = Box<dyn DictEntry>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl FromIterator<Box<dyn DictEntry>> for Lexicon {
    fn from_iter<T: IntoIterator<Item = Box<dyn DictEntry>>>(iter: T) -> Self {
        Self { entries: Vec::from_iter(iter) }
    }
}
