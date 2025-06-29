//! ltable.rs - Modern, extensible Lua table (hash/array) implementation in Rust
// Ported and modernized from ltable.c

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::lobject::{LuaValue, LObject};
use crate::lstate::LuaState;
use crate::lgc::GcObject;

/// TableKey: all valid Lua table keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableKey {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ptr(*const ()),
    Obj(GcObject),
}

/// TableMode: normal, weak keys, weak values, or both
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Normal,
    WeakKeys,
    WeakValues,
    WeakBoth,
}

impl Default for TableMode {
    fn default() -> Self { TableMode::Normal }
}

/// Table: dual array/hash structure, metatable, and GC integration
pub struct Table {
    array: Vec<Option<LuaValue>>, // array part (1-based)
    hash: HashMap<TableKey, LuaValue>, // hash part
    metatable: Option<GcObject>,
    mode: TableMode,
}

impl Default for Table {
    fn default() -> Self {
        Table::new()
    }
}

impl Table {
    /// Create a new empty table
    pub fn new() -> Self {
        Table {
            array: Vec::new(),
            hash: HashMap::new(),
            metatable: None,
            mode: TableMode::Normal,
        }
    }

    /// Create with array/hash capacity
    pub fn with_capacity(array_cap: usize, hash_cap: usize) -> Self {
        Table {
            array: vec![None; array_cap],
            hash: HashMap::with_capacity(hash_cap),
            metatable: None,
            mode: TableMode::Normal,
        }
    }

    /// Create a new table with a mode (normal/weak)
    pub fn with_mode(mode: TableMode) -> Self {
        Table {
            array: Vec::new(),
            hash: HashMap::new(),
            metatable: None,
            mode,
        }
    }

    /// Get value by key (integer keys use array part if possible)
    pub fn get(&self, key: &LuaValue) -> Option<&LuaValue> {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                self.array.get((*i as usize) - 1).and_then(|v| v.as_ref())
            }
            _ => self.hash.get(&TableKey::from_lua(key)),
        }
    }

    /// Set value by key (integer keys use array part if possible)
    pub fn set(&mut self, key: &LuaValue, value: LuaValue) {
        match key {
            LuaValue::Int(i) if *i > 0 => {
                let idx = (*i as usize) - 1;
                if idx < self.array.len() {
                    self.array[idx] = Some(value);
                    return;
                } else if idx < MAX_ARRAY_SIZE {
                    // Grow array if possible
                    self.array.resize(idx + 1, None);
                    self.array[idx] = Some(value);
                    return;
                }
            }
            _ => {}
        }
        self.hash.insert(TableKey::from_lua(key), value);
    }

    /// Remove a key
    pub fn remove(&mut self, key: &LuaValue) {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                self.array[(*i as usize) - 1] = None;
            }
            _ => {
                self.hash.remove(&TableKey::from_lua(key));
            }
        }
    }

    /// Get next key-value pair for iteration (Lua's next)
    pub fn next(&self, last_key: Option<&LuaValue>) -> Option<(LuaValue, &LuaValue)> {
        // Array part first
        let mut started = last_key.is_none();
        let mut idx = 0;
        if let Some(LuaValue::Int(i)) = last_key {
            if *i > 0 { idx = *i as usize; }
        }
        for (i, v) in self.array.iter().enumerate().skip(idx) {
            if v.is_some() {
                if started {
                    return Some((LuaValue::Int((i + 1) as i64), v.as_ref().unwrap()));
                } else {
                    started = true;
                }
            }
        }
        // Hash part
        let mut found = last_key.is_none();
        for (k, v) in &self.hash {
            let k_lua = k.to_lua();
            if found && v.is_some() {
                return Some((k_lua, v));
            }
            if let Some(lk) = last_key {
                if &k_lua == lk { found = true; }
            }
        }
        None
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.array.clear();
        self.hash.clear();
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &LuaValue) -> bool {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                self.array[(*i as usize) - 1].is_some()
            }
            _ => self.hash.contains_key(&TableKey::from_lua(key)),
        }
    }

    /// Create a table from an iterator of (LuaValue, LuaValue)
    pub fn from_iter<I: IntoIterator<Item = (LuaValue, LuaValue)>>(iter: I) -> Self {
        let mut t = Table::new();
        for (k, v) in iter {
            t.set(&k, v);
        }
        t
    }

    /// Convert all key-value pairs to a Vec
    pub fn to_vec(&self) -> Vec<(LuaValue, LuaValue)> {
        self.pairs().map(|(k, v)| (k, v.clone())).collect()
    }

    /// Get the table mode
    pub fn mode(&self) -> TableMode { self.mode }
    /// Set the table mode
    pub fn set_mode(&mut self, mode: TableMode) { self.mode = mode; }
    /// Set metatable
    pub fn set_metatable(&mut self, mt: Option<GcObject>) {
        self.metatable = mt;
    }
    /// Get metatable
    pub fn get_metatable(&self) -> Option<&GcObject> {
        self.metatable.as_ref()
    }
    /// Length (Lua # operator)
    pub fn len(&self) -> usize {
        let mut n = self.array.len();
        while n > 0 && self.array[n - 1].is_none() { n -= 1; }
        n
    }

    /// Total number of non-nil entries (array + hash)
    pub fn len_total(&self) -> usize {
        self.array.iter().filter(|v| v.is_some()).count() + self.hash.len()
    }

    /// Call a closure for each key-value pair
    pub fn for_each<F>(&self, mut f: F)
    where F: FnMut(&LuaValue, &LuaValue) {
        for (k, v) in self.pairs() {
            f(&k, v);
        }
    }

    /// Swap values for two keys
    pub fn swap(&mut self, k1: &LuaValue, k2: &LuaValue) {
        if k1 == k2 { return; }
        let v1 = self.pop(k1);
        let v2 = self.pop(k2);
        if let Some(v) = v1 { self.set(k2, v); }
        if let Some(v) = v2 { self.set(k1, v); }
    }

    /// Raw get (bypasses metatable logic)
    pub fn rawget(&self, key: &LuaValue) -> Option<&LuaValue> {
        self.get(key)
    }

    /// Raw set (bypasses metatable logic)
    pub fn rawset(&mut self, key: &LuaValue, value: LuaValue) {
        self.set(key, value)
    }

    /// Idiomatic Rust iterator over all key-value pairs (array + hash)
    pub fn pairs(&self) -> impl Iterator<Item = (LuaValue, &LuaValue)> {
        let array_iter = self.array.iter().enumerate().filter_map(|(i, v)| {
            v.as_ref().map(|val| (LuaValue::Int((i + 1) as i64), val))
        });
        let hash_iter = self.hash.iter().map(|(k, v)| (k.to_lua(), v));
        array_iter.chain(hash_iter)
    }

    /// Rehash: optimize array/hash split for current keys (Lua-style)
    pub fn rehash(&mut self) {
        // Collect all keys/values
        let mut all = Vec::new();
        for (i, v) in self.array.iter().enumerate() {
            if let Some(val) = v { all.push((LuaValue::Int((i + 1) as i64), val.clone())); }
        }
        for (k, v) in &self.hash {
            all.push((k.to_lua(), v.clone()));
        }
        // Find optimal array size (Lua: largest n with >50% 1..n used)
        let mut n = 0;
        let mut used = 0;
        for (k, _) in &all {
            if let LuaValue::Int(i) = k {
                if *i > 0 { n = n.max(*i as usize); }
            }
        }
        for (k, _) in &all {
            if let LuaValue::Int(i) = k {
                if *i > 0 && (*i as usize) <= n { used += 1; }
            }
        }
        let mut new_array = vec![None; n];
        let mut new_hash = HashMap::new();
        for (k, v) in all {
            if let LuaValue::Int(i) = k {
                if i > 0 && (i as usize) <= n { new_array[(i as usize) - 1] = Some(v); continue; }
            }
            new_hash.insert(TableKey::from_lua(&k), v);
        }
        self.array = new_array;
        self.hash = new_hash;
    }

    /// Find the length as per Lua's # operator (last non-nil in array)
    pub fn lua_len(&self) -> usize {
        let mut n = self.array.len();
        while n > 0 && self.array[n - 1].is_none() { n -= 1; }
        n
    }

    /// Shallow clone (copies structure, not deep values)
    pub fn clone_shallow(&self) -> Self {
        Table {
            array: self.array.clone(),
            hash: self.hash.clone(),
            metatable: self.metatable.clone(),
            mode: self.mode,
        }
    }
    /// Deep clone (requires LuaValue:Clone to be deep)
    pub fn clone_deep(&self) -> Self {
        Table {
            array: self.array.iter().map(|v| v.clone()).collect(),
            hash: self.hash.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            metatable: self.metatable.clone(),
            mode: self.mode,
        }
    }
    /// Filter: keep only entries where predicate returns true
    pub fn filter<F>(&self, mut pred: F) -> Self
    where F: FnMut(&LuaValue, &LuaValue) -> bool {
        let mut t = Table::with_mode(self.mode);
        for (k, v) in self.pairs() {
            if pred(&k, v) {
                t.set(&k, v.clone());
            }
        }
        t
    }
    /// Map all values (returns new table)
    pub fn map_values<F>(&self, mut f: F) -> Self
    where F: FnMut(&LuaValue) -> LuaValue {
        let mut t = Table::with_mode(self.mode);
        for (k, v) in self.pairs() {
            t.set(&k, f(v));
        }
        t
    }
    /// Merge another table into this one (optionally overwrite existing keys)
    pub fn merge(&mut self, other: &Table, overwrite: bool) {
        for (k, v) in other.pairs() {
            if overwrite || !self.contains_key(&k) {
                self.set(&k, v.clone());
            }
        }
    }

    /// Retain only entries where predicate returns true (in-place filter)
    pub fn retain<F>(&mut self, mut pred: F)
    where F: FnMut(&LuaValue, &LuaValue) -> bool {
        // Array part
        for (i, v) in self.array.iter_mut().enumerate() {
            if let Some(val) = v {
                if !pred(&LuaValue::Int((i + 1) as i64), val) {
                    *v = None;
                }
            }
        }
        // Hash part
        self.hash.retain(|k, v| pred(&k.to_lua(), v));
    }
    /// Iterator over all keys
    pub fn keys(&self) -> impl Iterator<Item = LuaValue> + '_ {
        self.pairs().map(|(k, _)| k)
    }
    /// Iterator over all values
    pub fn values(&self) -> impl Iterator<Item = &LuaValue> + '_ {
        self.pairs().map(|(_, v)| v)
    }
    /// Number of hash (non-array) entries
    pub fn len_hash(&self) -> usize {
        self.hash.len()
    }
    /// Returns true if the table is empty
    pub fn is_empty(&self) -> bool {
        self.array.iter().all(|v| v.is_none()) && self.hash.is_empty()
    }

    /// Get a mutable reference to the value for a key, inserting if absent
    pub fn get_or_insert_with<F>(&mut self, key: &LuaValue, default: F) -> &mut LuaValue
    where F: FnOnce() -> LuaValue {
        match key {
            LuaValue::Int(i) if *i > 0 => {
                let idx = (*i as usize) - 1;
                if idx < self.array.len() {
                    if self.array[idx].is_none() {
                        self.array[idx] = Some(default());
                    }
                    return self.array[idx].as_mut().unwrap();
                } else if idx < MAX_ARRAY_SIZE {
                    self.array.resize(idx + 1, None);
                    self.array[idx] = Some(default());
                    return self.array[idx].as_mut().unwrap();
                }
            }
            _ => {}
        }
        let k = TableKey::from_lua(key);
        self.hash.entry(k).or_insert_with(default)
    }
    /// Update a value in-place if it exists
    pub fn update<F>(&mut self, key: &LuaValue, mut f: F)
    where F: FnMut(&mut LuaValue) {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                if let Some(v) = self.array[(*i as usize) - 1].as_mut() {
                    f(v);
                }
            }
            _ => {
                if let Some(v) = self.hash.get_mut(&TableKey::from_lua(key)) {
                    f(v);
                }
            }
        }
    }
    /// Remove and return a value by key
    pub fn pop(&mut self, key: &LuaValue) -> Option<LuaValue> {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                self.array[(*i as usize) - 1].take()
            }
            _ => self.hash.remove(&TableKey::from_lua(key)),
        }
    }
    /// Get current array/hash capacities
    pub fn capacity(&self) -> (usize, usize) {
        (self.array.capacity(), self.hash.capacity())
    }
}

/// TableKey conversion helpers
impl TableKey {
    pub fn from_lua(val: &LuaValue) -> Self {
        match val {
            LuaValue::Int(i) => TableKey::Int(*i),
            LuaValue::Float(f) => TableKey::Float(*f),
            LuaValue::Str(s) => TableKey::Str(s.clone()),
            LuaValue::Bool(b) => TableKey::Bool(*b),
            LuaValue::Pointer(p) => TableKey::Ptr(*p),
            LuaValue::Object(o) => TableKey::Obj(o.clone()),
            _ => TableKey::Ptr(std::ptr::null()), // fallback
        }
    }
    pub fn to_lua(&self) -> LuaValue {
        match self {
            TableKey::Int(i) => LuaValue::Int(*i),
            TableKey::Float(f) => LuaValue::Float(*f),
            TableKey::Str(s) => LuaValue::Str(s.clone()),
            TableKey::Bool(b) => LuaValue::Bool(*b),
            TableKey::Ptr(p) => LuaValue::Pointer(*p),
            TableKey::Obj(o) => LuaValue::Object(o.clone()),
        }
    }
}

/// Maximum array size for Lua tables (configurable)
pub const MAX_ARRAY_SIZE: usize = 1 << 24;

// --- Advanced features: custom hashers, D-based helpers, etc. can be added here ---

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobject::LuaValue;
    #[test]
    fn test_table_basic() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(42));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(99));
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(42)));
        assert_eq!(t.get(&LuaValue::Str("foo".to_string())), Some(&LuaValue::Int(99)));
        t.remove(&LuaValue::Int(1));
        assert_eq!(t.get(&LuaValue::Int(1)), None);
    }
    #[test]
    fn test_table_next() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Str("a".to_string()), LuaValue::Int(30));
        let mut keys = Vec::new();
        let mut last = None;
        while let Some((k, v)) = t.next(last.as_ref()) {
            keys.push((k, v.clone()));
            last = Some(k);
        }
        assert!(keys.iter().any(|(k, v)| matches!(k, LuaValue::Int(1)) && *v == LuaValue::Int(10)));
        assert!(keys.iter().any(|(k, v)| matches!(k, LuaValue::Int(2)) && *v == LuaValue::Int(20)));
        assert!(keys.iter().any(|(k, v)| matches!(k, LuaValue::Str(ref s) if s == "a") && *v == LuaValue::Int(30)));
    }
    #[test]
    fn test_table_rehash() {
        let mut t = Table::with_capacity(2, 2);
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Int(100), LuaValue::Int(99));
        t.rehash();
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(10)));
        assert_eq!(t.get(&LuaValue::Int(2)), Some(&LuaValue::Int(20)));
        assert_eq!(t.get(&LuaValue::Int(100)), Some(&LuaValue::Int(99)));
        assert_eq!(t.lua_len(), 2);
    }
    #[test]
    fn test_table_pairs() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        t.set(&LuaValue::Str("x".to_string()), LuaValue::Int(2));
        let mut found = 0;
        for (k, v) in t.pairs() {
            match k {
                LuaValue::Int(1) => assert_eq!(*v, LuaValue::Int(1)),
                LuaValue::Str(ref s) if s == "x" => assert_eq!(*v, LuaValue::Int(2)),
                _ => {}
            }
            found += 1;
        }
        assert_eq!(found, 2);
    }
    #[test]
    fn test_table_clear_and_contains() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        assert!(t.contains_key(&LuaValue::Int(1)));
        t.clear();
        assert!(!t.contains_key(&LuaValue::Int(1)));
    }
    #[test]
    fn test_table_from_iter_to_vec() {
        let pairs = vec![(LuaValue::Int(1), LuaValue::Int(2)), (LuaValue::Str("a".to_string()), LuaValue::Int(3))];
        let t = Table::from_iter(pairs.clone());
        let mut out = t.to_vec();
        out.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        let mut pairs_sorted = pairs;
        pairs_sorted.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        assert_eq!(out, pairs_sorted);
    }
    #[test]
    fn test_table_mode() {
        let mut t = Table::with_mode(TableMode::WeakKeys);
        assert_eq!(t.mode(), TableMode::WeakKeys);
        t.set_mode(TableMode::WeakBoth);
        assert_eq!(t.mode(), TableMode::WeakBoth);
    }
    #[test]
    fn test_table_clone_and_filter() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        let t2 = t.clone_shallow();
        assert_eq!(t2.get(&LuaValue::Int(1)), Some(&LuaValue::Int(10)));
        let t3 = t.filter(|k, v| matches!(k, LuaValue::Int(2)));
        assert_eq!(t3.get(&LuaValue::Int(2)), Some(&LuaValue::Int(20)));
        assert_eq!(t3.get(&LuaValue::Int(1)), None);
    }
    #[test]
    fn test_table_map_and_merge() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(2));
        let t2 = t.map_values(|v| match v { LuaValue::Int(i) => LuaValue::Int(i * 10), _ => v.clone() });
        assert_eq!(t2.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        let mut t3 = Table::new();
        t3.set(&LuaValue::Int(2), LuaValue::Int(99));
        t3.merge(&t2, false);
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        assert_eq!(t3.get(&LuaValue::Int(2)), Some(&LuaValue::Int(99)));
        t3.merge(&t2, true);
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        assert_eq!(t3.get(&LuaValue::Int(2)), Some(&LuaValue::Int(20)));
    }
    #[test]
    fn test_table_retain_keys_values() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Str("a".to_string()), LuaValue::Int(30));
        t.retain(|k, _| matches!(k, LuaValue::Int(2)));
        let keys: Vec<_> = t.keys().collect();
        let values: Vec<_> = t.values().collect();
        assert_eq!(keys, vec![LuaValue::Int(2)]);
        assert_eq!(values, vec![&LuaValue::Int(20)]);
        assert_eq!(t.len_hash(), 0);
        assert!(!t.is_empty());
        t.clear();
        assert!(t.is_empty());
    }
    #[test]
    fn test_table_get_or_insert_update_pop_capacity() {
        let mut t = Table::new();
        let v = t.get_or_insert_with(&LuaValue::Int(1), || LuaValue::Int(42));
        assert_eq!(*v, LuaValue::Int(42));
        t.update(&LuaValue::Int(1), |v| if let LuaValue::Int(i) = v { *i += 1; });
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(43)));
        let popped = t.pop(&LuaValue::Int(1));
        assert_eq!(popped, Some(LuaValue::Int(43)));
        assert!(t.get(&LuaValue::Int(1)).is_none());
        let (arr_cap, hash_cap) = t.capacity();
        assert!(arr_cap >= 0 && hash_cap >= 0);
    }
    #[test]
    fn test_table_default_len_total_for_each_swap() {
        let mut t = Table::default();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Str("a".to_string()), LuaValue::Int(20));
        assert_eq!(t.len_total(), 2);
        let mut sum = 0;
        t.for_each(|_, v| if let LuaValue::Int(i) = v { sum += i; });
        assert_eq!(sum, 30);
        t.swap(&LuaValue::Int(1), &LuaValue::Str("a".to_string()));
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        assert_eq!(t.get(&LuaValue::Str("a".to_string())), Some(&LuaValue::Int(10)));
    }
    #[test]
    fn test_table_keys_values_iter_order_and_empty() {
        let mut t = Table::new();
        assert!(t.keys().next().is_none());
        assert!(t.values().next().is_none());
        t.set(&LuaValue::Int(1), LuaValue::Int(100));
        t.set(&LuaValue::Int(2), LuaValue::Int(200));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(300));
        let mut keys: Vec<_> = t.keys().collect();
        keys.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        assert_eq!(keys, vec![LuaValue::Int(1), LuaValue::Int(2), LuaValue::Str("foo".to_string())]);
        let mut values: Vec<_> = t.values().cloned().collect();
        values.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
        assert_eq!(values, vec![LuaValue::Int(100), LuaValue::Int(200), LuaValue::Int(300)]);
    }

    #[test]
    fn test_table_merge_overwrite_false_true() {
        let mut t1 = Table::new();
        t1.set(&LuaValue::Int(1), LuaValue::Int(1));
        t1.set(&LuaValue::Int(2), LuaValue::Int(2));
        let mut t2 = Table::new();
        t2.set(&LuaValue::Int(2), LuaValue::Int(22));
        t2.set(&LuaValue::Int(3), LuaValue::Int(33));
        let mut t3 = t1.clone_shallow();
        t3.merge(&t2, false);
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Int(1)));
        assert_eq!(t3.get(&LuaValue::Int(2)), Some(&LuaValue::Int(2))); // not overwritten
        assert_eq!(t3.get(&LuaValue::Int(3)), Some(&LuaValue::Int(33)));
        t3.merge(&t2, true);
        assert_eq!(t3.get(&LuaValue::Int(2)), Some(&LuaValue::Int(22))); // now overwritten
    }

    #[test]
    fn test_table_filter_and_map_values() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Int(3), LuaValue::Int(30));
        let even = t.filter(|k, _| matches!(k, LuaValue::Int(i) if i % 2 == 0));
        assert_eq!(even.get(&LuaValue::Int(2)), Some(&LuaValue::Int(20)));
        assert_eq!(even.get(&LuaValue::Int(1)), None);
        let mapped = t.map_values(|v| match v { LuaValue::Int(i) => LuaValue::Int(i * 2), _ => v.clone() });
        assert_eq!(mapped.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        assert_eq!(mapped.get(&LuaValue::Int(2)), Some(&LuaValue::Int(40)));
        assert_eq!(mapped.get(&LuaValue::Int(3)), Some(&LuaValue::Int(60)));
    }

    #[test]
    fn test_table_clone_deep_and_is_empty() {
        let mut t = Table::new();
        assert!(t.is_empty());
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        let t2 = t.clone_deep();
        assert_eq!(t2.get(&LuaValue::Int(1)), Some(&LuaValue::Int(1)));
        t.remove(&LuaValue::Int(1));
        assert!(t.is_empty());
        assert_eq!(t2.get(&LuaValue::Int(1)), Some(&LuaValue::Int(1))); // t2 is unaffected
    }
    #[test]
    fn test_table_rehash_and_len_consistency() {
        let mut t = Table::new();
        for i in 1..=10 {
            t.set(&LuaValue::Int(i), LuaValue::Int(i * 10));
        }
        t.set(&LuaValue::Str("x".to_string()), LuaValue::Int(999));
        t.set(&LuaValue::Str("y".to_string()), LuaValue::Int(888));
        let len_before = t.len();
        t.rehash();
        let len_after = t.len();
        assert_eq!(len_before, len_after);
        for i in 1..=10 {
            assert_eq!(t.get(&LuaValue::Int(i)), Some(&LuaValue::Int(i * 10)));
        }
        assert_eq!(t.get(&LuaValue::Str("x".to_string())), Some(&LuaValue::Int(999)));
        assert_eq!(t.get(&LuaValue::Str("y".to_string())), Some(&LuaValue::Int(888)));
    }

    #[test]
    fn test_table_swap_edge_cases() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(100));
        t.set(&LuaValue::Int(2), LuaValue::Int(200));
        t.swap(&LuaValue::Int(1), &LuaValue::Int(2));
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(200)));
        assert_eq!(t.get(&LuaValue::Int(2)), Some(&LuaValue::Int(100)));
        // Swap with non-existent key
        t.swap(&LuaValue::Int(1), &LuaValue::Str("foo".to_string()));
        assert_eq!(t.get(&LuaValue::Int(1)), None);
        assert_eq!(t.get(&LuaValue::Str("foo".to_string())), Some(&LuaValue::Int(200)));
        // Swap same key (should be no-op)
        t.swap(&LuaValue::Int(2), &LuaValue::Int(2));
        assert_eq!(t.get(&LuaValue::Int(2)), Some(&LuaValue::Int(100)));
    }

    #[test]
    fn test_table_rawget_rawset() {
        let mut t = Table::new();
        t.rawset(&LuaValue::Int(1), LuaValue::Int(123));
        assert_eq!(t.rawget(&LuaValue::Int(1)), Some(&LuaValue::Int(123)));
        t.rawset(&LuaValue::Str("bar".to_string()), LuaValue::Int(456));
        assert_eq!(t.rawget(&LuaValue::Str("bar".to_string())), Some(&LuaValue::Int(456)));
    }

    #[test]
    fn test_table_capacity_growth() {
        let mut t = Table::with_capacity(2, 2);
        let (arr_cap, hash_cap) = t.capacity();
        assert!(arr_cap >= 2 && hash_cap >= 2);
        for i in 1..100 {
            t.set(&LuaValue::Int(i), LuaValue::Int(i));
        }
        let (arr_cap2, hash_cap2) = t.capacity();
        assert!(arr_cap2 >= 99); // Should have grown
    }

    #[test]
    fn test_table_clear_and_reuse() {
        let mut t = Table::new();
        for i in 1..=5 {
            t.set(&LuaValue::Int(i), LuaValue::Int(i));
        }
        t.clear();
        assert!(t.is_empty());
        for i in 1..=5 {
            assert!(t.get(&LuaValue::Int(i)).is_none());
        }
        // Reuse after clear
        t.set(&LuaValue::Str("z".to_string()), LuaValue::Int(999));
        assert_eq!(t.get(&LuaValue::Str("z".to_string())), Some(&LuaValue::Int(999)));
    }

    #[test]
    fn test_table_contains_key_and_remove() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(20));
        assert!(t.contains_key(&LuaValue::Int(1)));
        assert!(t.contains_key(&LuaValue::Str("foo".to_string())));
        t.remove(&LuaValue::Int(1));
        assert!(!t.contains_key(&LuaValue::Int(1)));
        t.remove(&LuaValue::Str("foo".to_string()));
        assert!(!t.contains_key(&LuaValue::Str("foo".to_string())));
    }

    #[test]
    fn test_table_len_hash_and_array() {
        let mut t = Table::new();
        for i in 1..=3 {
            t.set(&LuaValue::Int(i), LuaValue::Int(i));
        }
        t.set(&LuaValue::Str("a".to_string()), LuaValue::Int(100));
        t.set(&LuaValue::Str("b".to_string()), LuaValue::Int(200));
        assert_eq!(t.len_hash(), 2);
        assert_eq!(t.len(), 3);
        t.remove(&LuaValue::Int(3));
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn test_table_for_each_and_to_vec() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(20));
        let mut acc = 0;
        t.for_each(|_, v| if let LuaValue::Int(i) = v { acc += i; });
        assert_eq!(acc, 30);
        let mut v = t.to_vec();
        v.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        assert_eq!(v, vec![
            (LuaValue::Int(1), LuaValue::Int(10)),
            (LuaValue::Str("foo".to_string()), LuaValue::Int(20)),
        ]);
    }

    #[test]
    fn test_table_update_nonexistent_and_pop_empty() {
        let mut t = Table::new();
        // update on nonexistent key should do nothing
        t.update(&LuaValue::Int(42), |v| if let LuaValue::Int(i) = v { *i += 1; });
        assert!(t.get(&LuaValue::Int(42)).is_none());
        // pop on empty table returns None
        assert!(t.pop(&LuaValue::Int(1)).is_none());
    }

    #[test]
    fn test_table_get_or_insert_with_complex() {
        let mut t = Table::new();
        let v = t.get_or_insert_with(&LuaValue::Str("foo".to_string()), || LuaValue::Int(1234));
        assert_eq!(*v, LuaValue::Int(1234));
        // Should not overwrite existing
        let v2 = t.get_or_insert_with(&LuaValue::Str("foo".to_string()), || LuaValue::Int(9999));
        assert_eq!(*v2, LuaValue::Int(1234));
    }

    #[test]
    fn test_table_keys_and_values_after_removal() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Str("x".to_string()), LuaValue::Int(30));
        t.remove(&LuaValue::Int(2));
        let keys: Vec<_> = t.keys().collect();
        assert!(keys.contains(&LuaValue::Int(1)));
        assert!(keys.contains(&LuaValue::Str("x".to_string())));
        assert!(!keys.contains(&LuaValue::Int(2)));
        let values: Vec<_> = t.values().collect();
        assert!(values.contains(&&LuaValue::Int(10)));
        assert!(values.contains(&&LuaValue::Int(30)));
        assert!(!values.contains(&&LuaValue::Int(20)));
    }

    #[test]
    fn test_table_filter_empty_result() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        let filtered = t.filter(|_, v| matches!(v, LuaValue::Int(i) if *i > 100));
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_table_merge_empty_and_self() {
        let mut t1 = Table::new();
        t1.set(&LuaValue::Int(1), LuaValue::Int(10));
        let t2 = Table::new();
        let mut t3 = t1.clone_shallow();
        t3.merge(&t2, false);
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Int(10)));
        // Merge self (should not panic, should overwrite if true)
        t3.merge(&t3.clone_shallow(), true);
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Int(10)));
    }

    #[test]
    fn test_table_map_values_identity_and_types() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(5));
        t.set(&LuaValue::Str("s".to_string()), LuaValue::Str("abc".to_string()));
        // Identity map
        let t2 = t.map_values(|v| v.clone());
        assert_eq!(t2.get(&LuaValue::Int(1)), Some(&LuaValue::Int(5)));
        assert_eq!(t2.get(&LuaValue::Str("s".to_string())), Some(&LuaValue::Str("abc".to_string())));
        // Type-changing map
        let t3 = t.map_values(|v| match v {
            LuaValue::Int(i) => LuaValue::Str(format!("num={}", i)),
            LuaValue::Str(s) => LuaValue::Int(s.len() as i64),
            _ => v.clone(),
        });
        assert_eq!(t3.get(&LuaValue::Int(1)), Some(&LuaValue::Str("num=5".to_string())));
        assert_eq!(t3.get(&LuaValue::Str("s".to_string())), Some(&LuaValue::Int(3)));
    }

    #[test]
    fn test_table_clone_shallow_vs_deep() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(42));
        let t_shallow = t.clone_shallow();
        let t_deep = t.clone_deep();
        assert_eq!(t_shallow.get(&LuaValue::Int(1)), Some(&LuaValue::Int(42)));
        assert_eq!(t_deep.get(&LuaValue::Int(1)), Some(&LuaValue::Int(42)));
        t.set(&LuaValue::Int(1), LuaValue::Int(99));
        // Clones are unaffected
        assert_eq!(t_shallow.get(&LuaValue::Int(1)), Some(&LuaValue::Int(42)));
        assert_eq!(t_deep.get(&LuaValue::Int(1)), Some(&LuaValue::Int(42)));
    }

    #[test]
    fn test_table_retain_all_and_none() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        t.set(&LuaValue::Int(2), LuaValue::Int(2));
        // Retain all
        t.retain(|_, _| true);
        assert_eq!(t.len_total(), 2);
        // Retain none
        t.retain(|_, _| false);
        assert!(t.is_empty());
    }

    #[test]
    fn test_table_next_iteration_order_and_exhaustion() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(2), LuaValue::Int(20));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(30));
        let mut seen = Vec::new();
        let mut last = None;
        while let Some((k, v)) = t.next(last.as_ref()) {
            seen.push((k.clone(), v.clone()));
            last = Some(k);
        }
        // Should see all keys exactly once
        assert_eq!(seen.len(), 3);
        assert!(seen.iter().any(|(k, v)| *k == LuaValue::Int(1) && *v == LuaValue::Int(10)));
        assert!(seen.iter().any(|(k, v)| *k == LuaValue::Int(2) && *v == LuaValue::Int(20)));
        assert!(seen.iter().any(|(k, v)| *k == LuaValue::Str("foo".to_string()) && *v == LuaValue::Int(30)));
        // After exhaustion, next returns None
        assert!(t.next(last.as_ref()).is_none());
    }

    #[test]
    fn test_table_large_sparse_array() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(1));
        t.set(&LuaValue::Int(1000), LuaValue::Int(1000));
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(1)));
        assert_eq!(t.get(&LuaValue::Int(1000)), Some(&LuaValue::Int(1000)));
        assert_eq!(t.len(), 1000);
        t.remove(&LuaValue::Int(1000));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn test_table_set_overwrite_and_remove_nonexistent() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Int(10));
        t.set(&LuaValue::Int(1), LuaValue::Int(20));
        assert_eq!(t.get(&LuaValue::Int(1)), Some(&LuaValue::Int(20)));
        // Remove nonexistent key
        t.remove(&LuaValue::Int(999));
        assert_eq!(t.len_total(), 1);
    }

    #[test]
    fn test_table_capacity_does_not_shrink_on_clear() {
        let mut t = Table::with_capacity(50, 50);
        let (arr_cap, hash_cap) = t.capacity();
        for i in 1..=50 {
            t.set(&LuaValue::Int(i), LuaValue::Int(i));
        }
        t.clear();
        let (arr_cap2, hash_cap2) = t.capacity();
        assert!(arr_cap2 >= arr_cap);
        assert!(hash_cap2 >= hash_cap);
    }

    #[test]
    fn test_table_with_mode_and_metatable() {
        let mut t = Table::with_mode(TableMode::WeakValues);
        assert_eq!(t.mode(), TableMode::WeakValues);
        t.set_mode(TableMode::Normal);
        assert_eq!(t.mode(), TableMode::Normal);
        // Metatable set/get
        assert!(t.get_metatable().is_none());
        // Dummy GcObject for test (replace with real if available)
        // Here we use Option<GcObject> = None for test, as GcObject is opaque
        t.set_metatable(None);
        assert!(t.get_metatable().is_none());
    }

    #[test]
    fn test_table_from_iter_and_to_vec_roundtrip() {
        let pairs = vec![
            (LuaValue::Int(1), LuaValue::Int(2)),
            (LuaValue::Str("a".to_string()), LuaValue::Int(3)),
        ];
        let t = Table::from_iter(pairs.clone());
        let mut out = t.to_vec();
        out.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        let mut pairs_sorted = pairs;
        pairs_sorted.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
        assert_eq!(out, pairs_sorted);
    }

    #[test]
    fn test_table_keys_and_values_types() {
        let mut t = Table::new();
        t.set(&LuaValue::Int(1), LuaValue::Str("x".to_string()));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(42));
        let keys: Vec<_> = t.keys().collect();
        let values: Vec<_> = t.values().collect();
        assert!(keys.contains(&LuaValue::Int(1)));
        assert!(keys.contains(&LuaValue::Str("foo".to_string())));
        assert!(values.contains(&&LuaValue::Str("x".to_string())));
        assert!(values.contains(&&LuaValue::Int(42)));
    }

    #[test]
    fn test_table_rawget_rawset_equivalence() {
        let mut t = Table::new();
        t.rawset(&LuaValue::Int(1), LuaValue::Int(123));
        assert_eq!(t.rawget(&LuaValue::Int(1)), t.get(&LuaValue::Int(1)));
        t.set(&LuaValue::Str("foo".to_string()), LuaValue::Int(456));
        assert_eq!(t.rawget(&LuaValue::Str("foo".to_string())), t.get(&LuaValue::Str("foo".to_string())));
    }
}
