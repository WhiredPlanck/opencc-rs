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
            DictEntry::NoValue { key: other.key() }
        } else if values.len() == 1 {
            DictEntry::StrSingleValue {
                key: other.key(),
                value: values[0].clone(),
            }
        } else {
            DictEntry::StrMultiValue {
                key: other.key(),
                values,
            }
        }
    }

    pub fn key(&self) -> String {
        match self {
            DictEntry::NoValue { key } => key.clone(),
            DictEntry::StrSingleValue { key, .. } => key.clone(),
            DictEntry::StrMultiValue { key, .. } => key.clone(),
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
        match self {
            DictEntry::NoValue { key } => key.clone(),
            DictEntry::StrSingleValue { key: _, value } => value.clone(),
            DictEntry::StrMultiValue { key, values } => {
                if values.is_empty() {
                    key.clone()
                } else {
                    values[0].clone()
                }
            }
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
