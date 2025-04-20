use dashmap::DashMap;
use std::hash::Hash;


pub struct Record<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone
{
    id: usize,
    data: Vec<DashMap<K, V>>
}

impl <K, V> Record<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone
{
    pub fn new (id: usize, size: usize) -> Self {

        let mut data = Vec::new();

        for _ in 0..size {
            data.push(DashMap::new())
        }

        Record {
            id,
            data,
        }
    }

    pub fn get_id(&self) -> usize { self.id }
    pub fn get_data_size(&self) -> usize { self.data.len() }
    pub fn is_data_empty(&self) -> bool { self.data.is_empty() }

    pub fn insert_data(&mut self, index: u32, key: K, value: V) -> Option<V> {

        self.data[index as usize].insert(key, value)

    }

    pub fn get_data_for_key(&self, index: u32, key: &K) -> Option<V> {

        self.data[index as usize].get(key).map(|v| v.clone())
    }

    pub fn get_a_map_in_vector(&self, index: u32) -> DashMap<K, V> {

        self.data[index as usize].clone()

    }

    pub fn get_data(&self) -> Vec<DashMap<K, V>> {

        self.data.clone()
    }

    pub fn get_all_keys(&self) -> Vec<K> {

        let mut keys = Vec::new();

        for dm in &self.data {
            for entry in dm.iter() {
                keys.push(entry.key().clone())

            }
        }
        keys
    }

    pub fn find_map_with_key(&self, key: &K) -> Option<&DashMap<K, V>> {

        self.data.iter().find(|map| map.contains_key(key))
    }

    pub fn modify_a_map_with_key<F>(&mut self, key: &K, operation: F) -> Option<&DashMap<K, V>>
    where
        F: FnOnce(&mut DashMap<K, V>) -> ()
    {
        // Find the index of the map containing the key
        let map_index = self.data.iter().position(|map| map.contains_key(key))?;

        // Apply the operation to the map
        operation(&mut self.data[map_index]);

        Some(&self.data[map_index])
    }

}