use std::{
    cmp::{max, min},
    collections::HashMap,
    fs::File,
    io::{Read, Write}, sync::Arc,
};

use rsmarisa::{
    Agent, Keyset, Trie,
    grimoire::io::{Reader, Writer},
};

use crate::{
    Dict, DictEntry, Error, Lexicon, SerializableDict, SerializedValues
};

static OCD2_HEADER: &str = "OPENCC_MARISA_0.2.5";

pub struct MarisaDict {
    max_length: usize,
    lexicon: Arc<Lexicon>,
    marisa: Trie,
}

impl MarisaDict {
    pub fn from_dict(dict: &dyn Dict) -> Self {
        let that_lexicon = &dict.lexicon();
        let mut max_key_length = 0;
        let mut keyset = Keyset::new();

        let mut key_value_map = HashMap::new();
        for entry in that_lexicon.iter() {
            keyset.push_back_str(entry.key().as_str()).unwrap();
            key_value_map.insert(entry.key(), DictEntry::new_from_other(&entry));
            max_key_length = max(entry.key().len(), max_key_length);
        }
        // Build Marisa Trie
        let mut marisa = Trie::new();
        marisa.build(&mut keyset, 0);
        let mut agent = Agent::new();
        agent.set_key_str("");
        let mut entries = Vec::new();
        entries.resize_with(that_lexicon.len(), || DictEntry::new_with_key(""));
        while marisa.predictive_search(&mut agent) {
            let key = String::from(agent.key().as_str());
            if let Some(entry) = key_value_map.remove(&key) {
                // Don't use `insert` here, it's slow
                entries[agent.key().id()] = entry;
            }
        }
        let lexicon = Arc::new(Lexicon::from_entries(entries));
        Self {
            max_length: max_key_length,
            lexicon,
            marisa,
        }
    }
}

impl Dict for MarisaDict {
    fn key_max_length(&self) -> usize {
        self.max_length
    }

    fn lexicon(&self) -> Arc<Lexicon> {
        self.lexicon.clone()
    }

    fn identity(&self) -> usize {
        self as *const Self as usize
    }

    fn match_word(&self, word: &str) -> Option<&DictEntry> {
        if word.len() > self.max_length {
            return None;
        }
        let lexicon = &self.lexicon;
        let mut agent = Agent::new();
        agent.set_query_str(word);
        if self.marisa.lookup(&mut agent) {
            Some(lexicon.get(agent.key().id()))
        } else {
            None
        }
    }

    fn match_prefix(&self, word: &str) -> Option<&DictEntry> {
        let mut agent = Agent::new();
        agent.set_query_str(&word[..min(self.max_length, word.len())]);
        let mut matched = None;
        while self.marisa.common_prefix_search(&mut agent) {
            matched = Some(self.lexicon.get(agent.key().id()));
        }
        matched
    }

    fn match_all_prefix(&self, word: &str) -> Vec<&DictEntry> {
        let mut agent = Agent::new();
        agent.set_query_str(&word[..min(self.max_length, word.len())]);
        let mut matches = Vec::new();
        while self.marisa.common_prefix_search(&mut agent) {
            let value = self.lexicon.get(agent.key().id());
            matches.push(value);
        }
        matches.reverse();
        matches
    }
}

impl SerializableDict for MarisaDict {
    fn new_from_file(file: &mut File) -> Result<Arc<dyn Dict>, Error> {
        let header_len: usize = OCD2_HEADER.len();
        let mut buffer = vec![0u8; header_len];
        if file.read_exact(&mut buffer).is_err() || str::from_utf8(&buffer)? != OCD2_HEADER
        {
            return Err(Error::InvalidFormat(
                "Invalid OpenCC dictionary header".to_string(),
            ));
        }
        let mut marisa = Trie::new();
        let file_clone = file.try_clone()?;
        let mut reader = Reader::from_reader(file_clone);
        marisa.read(&mut reader)?;
        let serialized_values = SerializedValues::new_from_file(file)?;
        let value_lexicon = &serialized_values.lexicon();
        let mut agent = Agent::new();
        agent.init_state().unwrap();
        agent.set_query_str("");
        let mut entries: Vec<DictEntry> = Vec::new();
        entries.resize_with(value_lexicon.len(), || DictEntry::new_with_key(""));
        let mut max_length = 0;
        while marisa.predictive_search(&mut agent) {
            let key = agent.key().as_str();
            let id = agent.key().id();
            max_length = max(key.len(), max_length);
            let entry = DictEntry::new_with_key_and_values(key, value_lexicon.get(id).values());
            // Don't use `insert` here, it's slow
            entries[id] = entry;
        }
        let lexicon = Arc::new(Lexicon::from_entries(entries));
        Ok(Arc::new(Self {
            max_length,
            lexicon,
            marisa,
        }))
    }

    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error> {
        file.write_all(OCD2_HEADER.as_bytes())?;
        let file_clone = file.try_clone()?;
        let mut writer = Writer::from_writer(file_clone);
        self.marisa.write(&mut writer)?;
        let serialized_values = SerializedValues::from_lexicon(self.lexicon.clone());
        serialized_values.serialize_to_file(file)?;
        Ok(())
    }
}
