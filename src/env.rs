use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Env<K, V> {
    tables: Vec<HashMap<K, V>>
}

impl<K, V> From<Vec<HashMap<K, V>>> for Env<K, V> {
    fn from(tables: Vec<HashMap<K, V>>) -> Self {
        Self {
            tables,
        }
    }
}

impl<K, V> Env<K, V> {
    pub fn push_empty(&mut self) {
        self.tables.push(HashMap::new());
    }

    pub fn push(&mut self, table: HashMap<K, V>) {
        self.tables.push(table)
    }

    pub fn pop(&mut self) -> HashMap<K, V> {
        self.tables.pop().unwrap()
    }
}

impl<K, V> Env<K, V> where K: Eq + Hash {
    pub fn put(&mut self, k: K, v: V) -> Option<V> {
        self.tables.last_mut().unwrap().insert(k, v)
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized {
        self.tables.iter().rev().find_map(|t| t.get(k))
    }
}
