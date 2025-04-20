use crossbeam::channel::{Receiver, Sender};
use rayon::prelude::*;
use std::hash::Hash;
use std::sync::Arc;
use dashmap::DashMap;
use crate::data_structures::MyData;

/// Different operation types that can be performed on the data structure
pub enum Operation<K, V> {
    Insert(K, V),
    Remove(K),
    Get(K, Sender<Option<V>>),
    Find(Arc<dyn Fn(&K, &V) -> bool + Send + Sync>, Sender<Vec<(K, V)>>),
    Clear,
    Shutdown,
}

/// Process a batch of keys in parallel using Rayon
pub fn process_keys_parallel<K, V, F, R>(
    data: &MyData<K, V>,
    keys: Vec<K>,
    processor: F,
) -> Vec<R>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(&K, Option<&V>) -> R + Send + Sync,
    R: Send,
{
    // Use Rayon's parallel iterator to process keys concurrently
    keys.par_iter().map(|key| {
        let value_ref = data.get(key);
        let value_option = value_ref.as_ref().map(|r| r.value());
        processor(key, value_option)
    }).collect()
}

/// Create a worker function that processes operations from a channel
pub fn create_worker_fn<K, V>(
    data: Arc<MyData<K, V>>,
    receiver: Receiver<Operation<K, V>>,
) -> impl FnOnce() -> ()
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    move || {
        // Process operations until shutdown signal is received
        for operation in receiver {
            match operation {
                Operation::Insert(key, value) => {
                    data.insert(key, value);
                }
                Operation::Remove(key) => {
                    data.remove(&key);
                }
                Operation::Get(key, sender) => {
                    let value = data.get(&key).map(|r| r.value().clone());
                    let _ = sender.send(value);
                }
                Operation::Find(predicate, sender) => {
                    let results = data.find(|k, v| predicate(k, v));
                    let _ = sender.send(results);
                }
                Operation::Clear => {
                    data.clear();
                }
                Operation::Shutdown => {
                    break;
                }
            }
        }
    }
}

/// Process all segments of the data structure in parallel using Rayon
pub fn parallel_segment_process<K, V, F>(
    data: &MyData<K, V>,
    processor: F,
)
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(&DashMap<K, V>) + Send + Sync + Clone,
{
    (0..data.num_segments())
        .into_par_iter()
        .for_each(|idx| {
            if let Some(segment) = data.get_segment(idx) {
                processor(segment);
            }
        });
}

/// Batch process multiple data structures in parallel
pub fn batch_process_parallel<K, V, F>(
    data_structures: &[Arc<MyData<K, V>>],
    batch_processor: F,
)
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(&MyData<K, V>) + Send + Sync + Clone,
{
    data_structures
        .par_iter()
        .for_each(|data| {
            batch_processor(data);
        });
}

/// Use Crossbeam scopes to process data with stack references
pub fn scoped_data_processing<K, V, F, R>(
    data: &MyData<K, V>,
    processor: F,
) -> Vec<R>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(usize, &DashMap<K, V>) -> R + Send + Sync + Clone,
    R: Send + 'static,
{
    let mut results = Vec::with_capacity(data.num_segments());

    crossbeam::scope(|scope| {
        let mut handles = Vec::with_capacity(data.num_segments());

        // Process each segment in its own thread
        for idx in 0..data.num_segments() {
            if let Some(segment) = data.get_segment(idx) {
                let processor_clone = processor.clone();
                let handle = scope.spawn(move |_| {
                    processor_clone(idx, segment)
                });
                handles.push(handle);
            }
        }

        // Collect results
        for handle in handles {
            if let Ok(result) = handle.join() {
                results.push(result);
            }
        }
    }).unwrap();

    results
}