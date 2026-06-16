use std::io::{BufRead, BufReader, Read};

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
    entries: Vec<DictEntry>
}

impl Lexicon {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn from_entries(entries: Vec<DictEntry>) -> Self {
        Self { entries }
    }

    pub fn add(&mut self, entry: DictEntry) {
        self.entries.push(entry);
    }

    pub fn sort(&mut self) {
        self.entries.sort();
    }

    pub fn is_sorted(&self) -> bool {
        self.entries.is_sorted_by(|a, b| a.key() < b.key())
    }

    pub fn is_unique(&self, dupkey: Option<&mut String>) -> bool {
        let entries = &self.entries;
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

    pub fn get(&self, index: usize) -> &DictEntry {
        &self.entries[index]
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, DictEntry> {
        self.entries.iter()
    }

    pub fn partition_point<P>(&self, pred: P)-> usize
    where
        P: FnMut(&DictEntry) -> bool,
    {
        self.entries.partition_point(pred)
    }

    pub fn parse_lexicon_from<R: Read>(reader: R) -> Result<Lexicon, Error> {
        let mut entries = Vec::new();
        let reader = BufReader::new(reader);
        for (i, line) in reader.lines().enumerate() {
            let line_num = i + 1;
            match parse_key_values(&line?, line_num) {
                Ok(entry) => entries.push(entry),
                Err(e) => return Err(e)
            }
        }
        Ok(Lexicon::from_entries(entries))
    }
}

impl FromIterator<DictEntry> for Lexicon {
    fn from_iter<T: IntoIterator<Item = DictEntry>>(iter: T) -> Self {
        Self { entries: Vec::from_iter(iter) }
    }
}
