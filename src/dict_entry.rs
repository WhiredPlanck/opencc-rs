pub trait DictEntry {
    fn key(&self) -> &str;
    fn value(&self) -> Option<&str>;
    fn values(&self) -> Vec<String>;
    fn get_default(&self) -> &str;
    fn to_string(&self) -> String;
}

impl Ord for dyn DictEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(other.key())
    }
}

impl PartialOrd for dyn DictEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.key().cmp(other.key()))
    }
}

impl PartialEq for dyn DictEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for dyn DictEntry {}

pub struct NoValueDictEntry {
    key: String
}

impl NoValueDictEntry {
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string() }
    }
}

impl DictEntry for NoValueDictEntry {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> Option<&str> {
        None
    }
    
    fn values(&self) -> Vec<String> {
        Vec::new()
    }
    
    fn get_default(&self) -> &str {
        &self.key
    }
    
    fn to_string(&self) -> String {
        self.key.clone()
    }
}

pub struct StrSingleValueDictEntry {
    key: String,
    value: String
}

impl DictEntry for StrSingleValueDictEntry {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> Option<&str> {
        Some(&self.value)
    }

    fn values(&self) -> Vec<String> {
        vec![self.value.clone()]
    }

    fn get_default(&self) -> &str {
        &self.value
    }

    fn to_string(&self) -> String {
        format!("{}\t{}", self.key, self.value)
    }
}

pub struct StrMultiValueDictEntry {
    key: String,
    values: Vec<String>
}

impl DictEntry for StrMultiValueDictEntry {
    fn key(&self) -> &str {
        &self.key
    }

    fn value(&self) -> Option<&str> {
        None
    }

    fn values(&self) -> Vec<String> {
        self.values.clone()
    }

    fn get_default(&self) -> &str {
        if self.values.is_empty() {
            &self.key
        } else {
            &self.values[0]
        }
    }

    fn to_string(&self) -> String {
        format!("{}\t{}", self.key, self.values.join(" "))
    }
}

pub struct DictEntryFactory;

impl DictEntryFactory {
    pub fn new_with_key(key: &str) -> Box<dyn DictEntry> {
        Box::new(NoValueDictEntry { key: key.to_string() })
    }

    pub fn new_with_key_and_value(key: &str, value: &str) -> Box<dyn DictEntry> {
        Box::new(
            StrSingleValueDictEntry { 
                key: key.to_string(),
                value: value.to_string()
            }
        )
    }

    pub fn new_with_key_and_values(key: &str, values: Vec<String>) -> Box<dyn DictEntry> {
        if values.is_empty() {
            DictEntryFactory::new_with_key(key)
        } else if values.len() == 1 {
            DictEntryFactory::new_with_key_and_value(key, &values[0])
        } else {
            Box::new(
                StrMultiValueDictEntry {
                    key: key.to_string(),
                    values: values
                }
            )
        }
    }

    pub fn new_from_other(other: &dyn DictEntry) -> Box<dyn DictEntry> {
        let values = other.values();
        if values.is_empty() {
            Box::new(NoValueDictEntry { key: other.key().to_owned() })
        } else if values.len() == 1 {
            Box::new(StrSingleValueDictEntry {
                key: other.key().to_owned(),
                value: values[0].clone()
            })
        } else {
            Box::new(StrMultiValueDictEntry {
                key: other.key().to_owned(),
                values: values
            })
        }

    }
}

impl Default for Box<dyn DictEntry> {
    fn default() -> Self {
        DictEntryFactory::new_with_key("")
    }
}
