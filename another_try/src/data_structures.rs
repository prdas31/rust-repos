use std::collections::HashMap;

use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub struct MyDataMaps <K, V>{
    inner: Vec<HashMap<K, Arc<Mutex<V>>>>,
}



impl <K, V> MyDataMaps<K, V>
where
    K: Eq + Hash
{
    pub fn new() -> Self {

        let mut result = Self {
            inner: Vec::new()
        };

        result.inner.push(HashMap::new());

        result
    }

    pub fn get_inner_vector(&self, index: usize) -> Option<&HashMap<K, Arc<Mutex<V>>>>
    {
        self.inner.get(index)
    }

    pub fn get_inner_hash_map_value(&self, index: usize, key: &K) -> Option<Arc<Mutex<V>>> {

        self.get_inner_vector(index).and_then(|hm| hm.get(key).cloned())
    }



    pub fn set_initial_values_for_internal_hash_maps(&mut self, index: usize, key: K, value: V)
    where
        K: Eq + Hash + Clone,
    {
        // No need to call get_inner_value_of_key here

        if let Some(hash_map) = self.inner.get_mut(index) {
            hash_map.insert(key, Arc::new(Mutex::new(value)));
        }
    }

    pub fn modify_value_in_vec_inside_hash_map_with_key<F> (&self, index: usize, key: &K, mut modifier: F) -> Option<()>
    where
        K: Eq + Hash + Clone,
        V: std::ops::AddAssign<V> + Clone,
        F: FnMut(&mut V),
    {
        /*
        let mutex_arc  = self.get_inner_value_of_key(index, key)?;

        match mutex_arc.lock() {

            Ok(mut guard_val) => {

                let clone_val = guard_val.clone();
                *guard_val += clone_val;

                Some(())

            },
            Err(_) => None, // Handle poisoned mutex
        } */

        // Get the Arc<Mutex<V>>
        let arc_mutex = self.get_inner_hash_map_value(index, key)?;

        // Create a separate variable for the lock result to extend its lifetime
        let lock_result = arc_mutex.lock();

        match lock_result {
            Ok(mut guard_val) => {
                // Apply the provided closure to modify the value
                modifier(&mut *guard_val);
                Some(())
            },
            Err(_) => None, // Handle poisoned mutex
        }
    }
}

impl<K, V> Clone for MyDataMaps<K, V>
where
    K: Clone + Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}