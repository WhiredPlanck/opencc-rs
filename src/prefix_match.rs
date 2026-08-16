use std::{collections::HashMap, fmt::Write, sync::{Arc, Weak}};

use dashmap::DashMap;
use once_cell::sync::OnceCell;

use crate::{AnyDict, Dict, DictGroupMatchPolicy, PrefixMatchResult};

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

enum Matcher {
    Leaf(LeafMatcher),
    Group(GroupMatcher)
}

impl Matcher {
    fn match_prefix(&self, word: &str) -> Option<PrefixMatchResult<'_>> {
        match self {
            Matcher::Leaf(matcher) => matcher.match_prefix(word),
            Matcher::Group(matcher) => match matcher.match_policy {
                DictGroupMatchPolicy::ShortCircuit => matcher.match_prefix_short_circuit(word),
                DictGroupMatchPolicy::Union => matcher.match_prefix_union(word)
            }
        }
    } 
}

#[derive(Default)]
struct LeafMatcher {
    root: Node
}

impl LeafMatcher {
    fn from_dict(dict: &Arc<AnyDict>) -> Self {
        let mut matcher = LeafMatcher::default();
        matcher.add_dict(dict);
        matcher
    }

    fn add_dict(&mut self, dict: &Arc<AnyDict>) {
        let lexicon = dict.lexicon();
        for entry in lexicon.iter() {
            self.add_entry(&entry.key(), &entry.get_default());
        }
    }

    fn add_entry(&mut self, key: &str, value: &str) {
        let mut node = &mut self.root;
        for ch in key.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.key = key.to_owned();
        node.value = value.to_owned();
    }

    fn match_prefix(&self, word: &str) -> Option<PrefixMatchResult<'_>> {
        let mut node = &self.root;
        let mut last_match: Option<&Node> = None;

        for ch in word.chars() {
            let next = match node.children.get(&ch) {
                Some(next) => next,
                None => break,
            };
            node = next;
            last_match = Some(node);
        }

        last_match.map(|term| PrefixMatchResult::new(&term.key, &term.value))
    }
}

struct GroupMatcher {
    children: Vec<Matcher>,
    match_policy: DictGroupMatchPolicy
}

impl GroupMatcher {
    fn new(match_policy: DictGroupMatchPolicy) -> Self {
        Self { children: Vec::new(), match_policy } 
    }

    fn add_child(&mut self, matcher: Matcher) {
        self.children.push(matcher);
    }

    fn match_prefix_short_circuit(&self, word: &str) -> Option<PrefixMatchResult<'_>> {
        self.children.iter().find_map(|child| child.match_prefix(word))
    }

    fn match_prefix_union(&self, word:&str) -> Option<PrefixMatchResult<'_>> {
        self.children.iter().filter_map(|child| child.match_prefix(word))
            .fold(None, |acc, candidate| {
                match acc {
                    Some(best) => {
                        if candidate.key().len() > best.key().len() {
                            Some(candidate)
                        } else {
                            Some(best)
                        }
                    },
                    None => Some(candidate)
                }
            })
    }
}

pub struct Tables {
    matcher: Matcher
}

pub struct PrefixMatch {
    tables: Option<Arc<Tables>>,
    single_dict: Option<Arc<AnyDict>>
}

fn same_dicts(cached: &[Weak<AnyDict>], current: &[Weak<AnyDict>]) -> bool {
    if cached.len() != current.len() {
        return false;
    }

    cached.iter().zip(current.iter()).all(|(a, b)| {
        a.upgrade().is_some() && b.upgrade().is_some() && Weak::ptr_eq(a, b)
    })
}

fn prune_expired_prefix_match_cache(cache: &DashMap<String, Vec<CacheEntry>>) {
    cache.retain(|_, entries| {
        entries.retain(|entry| !entry.has_expired_dict());
        !entries.is_empty()
    });
}

fn can_flatten_as_union(dict: &Arc<AnyDict>) -> bool {
    dict.dict_group_items().is_none_or(|items| {
        matches!(dict.match_policy(), DictGroupMatchPolicy::Union) ||
            items.iter().all(|child| can_flatten_as_union(child))
    })
}

fn collect_all_leaf_dicts(dict: &Arc<AnyDict>, out: &mut LeafMatcher) {
    if let Some(items) = dict.dict_group_items() {
        for child in items {
            collect_all_leaf_dicts(child, out);
        }
    } else {
        out.add_dict(dict);
    }
}

fn build_matcher(dict: &Arc<AnyDict>) -> Matcher {
    if let Some(dict_group_items) = dict.dict_group_items() {
        if can_flatten_as_union(dict) {
            let mut leaf = LeafMatcher::default();
            collect_all_leaf_dicts(dict, &mut leaf);
            return Matcher::Leaf(leaf);
        }
        
        let mut group = GroupMatcher::new(dict.match_policy());
        for child in dict_group_items {
            group.add_child(build_matcher(child));
        }
        return Matcher::Group(group);
    }
    Matcher::Leaf(LeafMatcher::from_dict(dict))
}

impl PrefixMatch {
    pub fn from_dict(dict: &Arc<AnyDict>) -> Self {
        let mut actual_dict = Some(dict);
        while let Some(actual) = actual_dict {
            if let Some(items) = actual.dict_group_items() && items.len() == 1 {
                actual_dict = items.first()
            } else {
                break;
            }
        }

        if let Some(actual) = actual_dict && actual.supports_fast_prefix_match() {
            return Self { tables: None, single_dict: Some(actual.clone()) }
        }

        static CACHE: OnceCell<DashMap<String, Vec<CacheEntry>>> = OnceCell::new();
        let cache = CACHE.get_or_init(|| DashMap::new());

        let mut cache_key = String::new();
        Self::append_cache_key(dict, &mut cache_key);
        let mut leave_dicts = Vec::new();
        Self::collect_leaf_dicts(dict, &mut leave_dicts);

        // try get cached tables
        {
            prune_expired_prefix_match_cache(&cache);
            let cached = cache.entry(cache_key.clone()).or_default();
            for entry in cached.iter() {
                if same_dicts(&entry.dicts, &leave_dicts) {
                    if let Some(tables) = entry.tables.upgrade() {
                        return Self { tables: Some(tables), single_dict: None };
                    }
                }
            }
        }

        let tables = Tables { matcher: build_matcher(&dict) };
        let built = Arc::new(tables);

        prune_expired_prefix_match_cache(&cache);
        let mut entries = cache.entry(cache_key).or_default();
        entries.retain(|entry| {
            !entry.has_expired_dict() || 
            (same_dicts(&entry.dicts, &leave_dicts) 
                && entry.tables.upgrade().is_none())
        });
        for entry in entries.iter() {
            if let Some(tables) = entry.tables.upgrade() {
                return Self { tables: Some(tables), single_dict: None }
            }
        }
        entries.push(CacheEntry {
            dicts: leave_dicts,
            tables: Arc::downgrade(&built)
        });
        Self { tables: Some(built), single_dict: None }
    }

    pub fn match_prefix(&self, word: &str) -> Option<PrefixMatchResult<'_>> {
        if let Some(single) = &self.single_dict {
            return single.match_prefix_value(word);
        }
        self.tables.as_ref().unwrap().matcher.match_prefix(word)
    }

    fn append_cache_key(dict: &Arc<AnyDict>, output: &mut String) {
        if let Some(dict_group_items) = dict.dict_group_items() {
            output.push('[');
            match dict.match_policy() {
                DictGroupMatchPolicy::ShortCircuit => output.push_str("short_circuit:"),
                DictGroupMatchPolicy::Union => output.push_str("union:"),
            }
            for child in dict_group_items {
                Self::append_cache_key(child, output);
            }
            output.push(']');
        } else {
            let dict_key = dict.identity();
            // `write!` avoids the temporary String that `to_string()` would allocate.
            let _ = write!(output, "{}{}", dict_key, ';');
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

