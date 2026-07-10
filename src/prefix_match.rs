use std::{cell::RefCell, collections::HashMap, sync::{Arc, RwLock, Weak}};

use once_cell::sync::OnceCell;

use crate::{AnyDict, Dict};

struct CacheEntry {
    dicts: Vec<Weak<AnyDict>>,
    tables: Weak<Tables>
}

impl CacheEntry {
    fn has_expired_dict(&self) -> bool {
        if self.tables.upgrade().is_none() {
            return true;
        }
        self.dicts.iter().any(|dict| dict.upgrade().is_none())
    }
}

#[derive(Default)]
struct Node {
    key: String,
    value: String,
    children: HashMap<char, Node>,
}

#[derive(Default)]
struct Table {
    root: Node
}

impl Table {
    fn from_dict(dict: &Arc<AnyDict>) -> Self {
        let lexicon = dict.lexicon();
        let mut table = Table::default();
        for entry in lexicon.iter() {
            table.add_entry(&entry.key(), &entry.get_default());
        }
        table
    }

    fn add_entry(&mut self, key: &str, value: &str) {
        let mut node = &mut self.root;
        for ch in key.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.key = key.to_owned();
        node.value = value.to_owned();
    }

    fn match_prefix(&self, word: &str) -> Option<MatchResult> {
        let mut node = &self.root;
        let mut last_match: Option<&Node> = None;

        for (_, ch) in word.char_indices() {
            let next = match node.children.get(&ch) {
                Some(next) => next,
                None => break,
            };
            node = next;
            last_match = Some(node);
        }

        last_match.map(|term| MatchResult {
            key: term.key.to_owned(),
            value: term.value.to_owned(),
        })
    }
}

pub struct Tables {
    tables: Vec<Table>
}

impl Tables {
    fn new() -> Self {
        Self { tables: Vec::new() }
    }
}

pub struct MatchResult {
    pub key: String,
    pub value: String
}

pub struct PrefixMatch {
    tables: Arc<Tables>
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Vec<CacheEntry>>> = RefCell::new(HashMap::new());
}

fn same_dicts(cached: &[Weak<AnyDict>], current: &[Weak<AnyDict>]) -> bool {
    if cached.len() != current.len() {
        return false;
    }

    cached.iter().zip(current.iter()).all(|(a, b)| {
        a.upgrade().is_some() && b.upgrade().is_some() && Weak::ptr_eq(a, b)
    })
}

fn prune_expired_prefix_match_cache(cache: &mut HashMap<String, Vec<CacheEntry>>) {
    cache.retain(|_, entries| {
        entries.retain(|entry| !entry.has_expired_dict());
        !entries.is_empty()
    });
}

impl PrefixMatch {
    pub fn from_dict(dict: &Arc<AnyDict>) -> Self {
        static CACHE: OnceCell<RwLock<HashMap<String, Vec<CacheEntry>>>> = OnceCell::new();
        let lock = CACHE.get_or_init(|| RwLock::new(HashMap::new()));

        let mut cache_key = String::new();
        Self::append_cache_key(dict, &mut cache_key);
        let mut leave_dicts = Vec::new();
        Self::collect_leaf_dicts(dict, &mut leave_dicts);

        // try get cached tables
        {
            let mut cache = lock.write().unwrap();
            prune_expired_prefix_match_cache(&mut cache);
            let cached = cache.entry(cache_key.clone()).or_default();
            for entry in cached {
                if same_dicts(&entry.dicts, &leave_dicts) {
                    if let Some(tables) = entry.tables.upgrade() {
                        return Self { tables };
                    }
                }
            }
        }

        let mut tables = Tables::new();
        Self::add_dict(&dict, &mut tables);
        let built = Arc::new(tables);

        let mut cache = lock.write().unwrap();
        prune_expired_prefix_match_cache(&mut cache);
        let entries = cache.entry(cache_key).or_default();
        entries.retain(|entry| {
            !entry.has_expired_dict() || 
            (same_dicts(&entry.dicts, &leave_dicts) 
                && entry.tables.upgrade().is_none())
        });
        for entry in entries.iter() {
            if let Some(tables) = entry.tables.upgrade() {
                return Self { tables }
            }
        }
        entries.push(CacheEntry {
            dicts: leave_dicts,
            tables: Arc::downgrade(&built)
        });
        Self { tables: built }
    }

    pub fn match_prefix(&self, word: &str) -> Option<MatchResult> {
        self.tables.tables.iter().find_map(|table| table.match_prefix(word))
    }

    fn add_dict(dict: &Arc<AnyDict>, output: &mut Tables) {
        if let Some(dict_group_items) = dict.dict_group_items() {
            for child in dict_group_items {
                Self::add_dict(child, output);
            }
        } else {
            output.tables.push(Table::from_dict(dict));
        }
    }

    fn append_cache_key(dict: &Arc<AnyDict>, output: &mut String) {
        if let Some(dict_group_items) = dict.dict_group_items() {
            output.push('[');
            for child in dict_group_items {
                Self::append_cache_key(child, output);
            }
            output.push(']');
        } else {
            let dict_key = dict.identity();
            output.push_str(&dict_key.to_string());
            output.push(';');
        }
    }

    fn collect_leaf_dicts(dict: &Arc<AnyDict>, out: &mut Vec<Weak<AnyDict>>) {
        if let Some(children) = dict.dict_group_items() {
            for child in children {
                Self::collect_leaf_dicts(child, out);
            }
        } else {
            out.push(Arc::downgrade(&dict));
        }
    }
}

