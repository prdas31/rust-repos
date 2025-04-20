use dashmap::DashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A thread-safe data structure that uses sharding to reduce contention
/// across multiple hashmap segments
pub struct MyData<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    id: usize,
    segments: Vec<DashMap<K, V>>,
    // Keep track of operations to help with monitoring
    op_counter: Arc<AtomicUsize>,
    // Store a consistent hasher for deterministic segment assignment
    hasher: std::collections::hash_map::RandomState,
}

impl<K, V> MyData<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new MyData instance with the specified number of segments
    pub fn new(id: usize, num_segments: usize) -> Self {
        let mut segments = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            segments.push(DashMap::new());
        }

        MyData {
            id,
            segments,
            op_counter: Arc::new(AtomicUsize::new(0)),
            hasher: std::collections::hash_map::RandomState::new(),
        }
    }

    /// Get the ID of this data structure
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the number of segments in this data structure
    pub fn num_segments(&self) -> usize {
        self.segments.len()
    }

    /// Get the total number of entries across all segments
    pub fn len(&self) -> usize {
        self.segments.iter().map(|segment| segment.len()).sum()
    }

    /// Check if all segments are empty
    pub fn is_empty(&self) -> bool {
        self.segments.iter().all(|segment| segment.is_empty())
    }

    /// Determine which segment a key belongs to
    fn get_segment_index(&self, key: &K) -> usize {
        // Use the consistent hasher stored in the struct
        let hash = std::hash::BuildHasher::hash_one(&self.hasher, key);
        (hash as usize) % self.segments.len()
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let segment_idx = self.get_segment_index(&key);
        let result = self.segments[segment_idx].insert(key, value);
        self.op_counter.fetch_add(1, Ordering::Relaxed);
        result
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<dashmap::mapref::one::Ref<K, V>> {
        let segment_idx = self.get_segment_index(key);
        self.segments[segment_idx].get(key)
    }

    /// Remove a key-value pair
    pub fn remove(&self, key: &K) -> Option<(K, V)> {
        let segment_idx = self.get_segment_index(key);
        let result = self.segments[segment_idx].remove(key);
        if result.is_some() {
            self.op_counter.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// Process each key-value pair with the given function
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&K, &mut V) + Send + Clone + 'static,
    {
        for segment in &self.segments {
            for mut entry in segment.iter_mut() {
                // Clone the key to avoid borrowing issues
                let key = entry.key().clone();
                // Now we can mutably borrow the value
                f(&key, entry.value_mut());
            }
            self.op_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get the operation counter
    pub fn op_count(&self) -> usize {
        self.op_counter.load(Ordering::Relaxed)
    }

    /// Execute a transaction that may involve multiple operations
    /// This ensures the operations are performed atomically on a single entry
    pub fn transaction<F, R>(&self, key: &K, transaction: F) -> Option<R>
    where
        F: FnOnce(&K, &mut V) -> R,
    {
        let segment_idx = self.get_segment_index(key);
        let segment = &self.segments[segment_idx];

        // Try to get a mutable reference to the entry
        if let Some(mut entry) = segment.get_mut(key) {
            // Clone the key to avoid borrowing issues
            let key_clone = entry.key().clone();
            // Now we can mutably borrow the value
            let result = transaction(&key_clone, entry.value_mut());
            self.op_counter.fetch_add(1, Ordering::Relaxed);
            Some(result)
        } else {
            None
        }
    }

    /// Clear all segments
    pub fn clear(&self) {
        for segment in &self.segments {
            segment.clear();
        }
        self.op_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Get all keys across all segments
    pub fn keys(&self) -> Vec<K> {
        let mut keys = Vec::new();
        for segment in &self.segments {
            for entry in segment.iter() {
                keys.push(entry.key().clone());
            }
        }
        keys
    }

    /// Create a clone of this data structure (clones all entries)
    pub fn clone_data(&self) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let new_segment = DashMap::new();
            for entry in segment.iter() {
                new_segment.insert(entry.key().clone(), entry.value().clone());
            }
            segments.push(new_segment);
        }

        MyData {
            id: self.id,
            segments,
            op_counter: Arc::new(AtomicUsize::new(self.op_count())),
            hasher: self.hasher.clone(),
        }
    }

    /// Get a specific segment for direct access
    /// This can be useful for batch operations on a segment
    pub fn get_segment(&self, segment_idx: usize) -> Option<&DashMap<K, V>> {
        if segment_idx < self.segments.len() {
            Some(&self.segments[segment_idx])
        } else {
            None
        }
    }

    /// Find entries that match a predicate
    pub fn find<F>(&self, predicate: F) -> Vec<(K, V)>
    where
        F: Fn(&K, &V) -> bool + Send + Sync + Clone,
    {
        let mut results = Vec::new();

        for segment in &self.segments {
            for entry in segment.iter() {
                if predicate(entry.key(), entry.value()) {
                    results.push((entry.key().clone(), entry.value().clone()));
                }
            }
        }

        results
    }
}

// Implementation for safe cloning of the entire structure
impl<K, V> Clone for MyData<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        self.clone_data()
    }
}