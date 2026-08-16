pub enum DictEntry {
    NoValue { key: String },
    StrSingleValue { key: String, value: String },
    StrMultiValue { key: String, values: Vec<String> },
}

impl DictEntry {
    pub fn new_with_key(key: &str) -> DictEntry {
        DictEntry::NoValue {
            key: key.to_string(),
        }
    }

    pub fn new_with_key_and_value(key: &str, value: &str) -> DictEntry {
        DictEntry::StrSingleValue {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn new_with_key_and_values(key: &str, values: Vec<String>) -> DictEntry {
        if values.is_empty() {
            DictEntry::new_with_key(key)
        } else if values.len() == 1 {
            DictEntry::new_with_key_and_value(key, &values[0])
        } else {
            DictEntry::StrMultiValue {
                key: key.to_string(),
                values,
            }
        }
    }

    pub fn new_from_other(other: &DictEntry) -> DictEntry {
        let values = other.values();
        if values.is_empty() {
            DictEntry::NoValue { key: other.key().to_string() }
        } else if values.len() == 1 {
            DictEntry::StrSingleValue {
                key: other.key().to_string(),
                value: values[0].clone(),
            }
        } else {
            DictEntry::StrMultiValue {
                key: other.key().to_string(),
                values,
            }
        }
    }

    /// Borrowed key, avoiding the allocation that the owned `String`
    /// accessor would incur (sorting, lookups and dictionary builds all
    /// read the key far more often than they need to own it).
    pub fn key(&self) -> &str {
        match self {
            DictEntry::NoValue { key } => key,
            DictEntry::StrSingleValue { key, .. } => key,
            DictEntry::StrMultiValue { key, .. } => key,
        }
    }

    pub fn value(&self) -> Option<String> {
        match self {
            DictEntry::NoValue { .. } | DictEntry::StrMultiValue { .. } => None,
            DictEntry::StrSingleValue { key: _, value } => Some(value.clone()),
        }
    }

    pub fn values(&self) -> Vec<String> {
        match self {
            DictEntry::NoValue { .. } => Vec::new(),
            DictEntry::StrSingleValue { key: _, value } => vec![value.clone()],
            DictEntry::StrMultiValue { key: _, values } => values.clone(),
        }
    }

    pub fn get_default(&self) -> String {
        self.get_default_ref().to_string()
    }

    /// Borrowed variant of [`get_default`](Self::get_default).
    pub fn get_default_ref(&self) -> &str {
        match self {
            DictEntry::NoValue { key } => key,
            DictEntry::StrSingleValue { key: _, value } => value,
            DictEntry::StrMultiValue { key, values } => values.first().map_or(key, |v| v),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DictEntry::NoValue { key } => key.clone(),
            DictEntry::StrSingleValue { key, value } => format!("{}\t{}", key, value),
            DictEntry::StrMultiValue { key, values } => format!("{}\t{}", key, values.join(" ")),
        }
    }
}

impl std::fmt::Display for DictEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictEntry::NoValue { key } => write!(f, "{}", key),
            DictEntry::StrSingleValue { key, value } => write!(f, "{}\t{}", key, value),
            DictEntry::StrMultiValue { key, values } => write!(f, "{}\t{}", key, values.join(" ")),
        }
    }
}

impl Ord for DictEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for DictEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.key().cmp(&other.key()))
    }
}

impl PartialEq for DictEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for DictEntry {}
