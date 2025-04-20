use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::hash::Hash;

pub struct ThreadSafeMap<K, V> {
    inner: Arc<RwLock<HashMap<K, V>>>
}

impl<K, V> ThreadSafeMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.write().unwrap().insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        self.inner.read().unwrap().get(key).cloned()
    }
}

impl<K, V> Clone for ThreadSafeMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner)
        }
    }
}