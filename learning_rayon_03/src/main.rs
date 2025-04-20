use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

// For Rayon example
use rayon::prelude::*;

// For concurrent collections
use dashmap::DashMap;

fn main() {

    let data: Vec<i32> = (0..1_000_000).map(|i| i % 100).collect();

    // Example: Build a frequency map
    println!("Running frequency map examples...");


    let start = Instant::now();
    let freq_sequential = frequency_map_sequential(&data);
    println!("Sequential frequency map with {} entries completed in {:?}",
             freq_sequential.len(), start.elapsed());

    let start = Instant::now();
    let freq_mutex = frequency_map_with_mutex(&data);
    println!("Mutex-based frequency map with {} entries completed in {:?}",
             freq_mutex.len(), start.elapsed());

    let start = Instant::now();
    let freq_rwlock = frequency_map_with_rwlock(&data);
    println!("RwLock-based frequency map with {} entries completed in {:?}",
             freq_rwlock.len(), start.elapsed());

    let start = Instant::now();
    let freq_dashmap = frequency_map_with_dashmap(&data);
    println!("DashMap-based frequency map with {} entries completed in {:?}",
             freq_dashmap.len(), start.elapsed());

    let start = Instant::now();
    let freq_rayon_fold = frequency_map_with_rayon_fold(&data);
    println!("Rayon fold-based frequency map with {} entries completed in {:?}",
             freq_rayon_fold.len(), start.elapsed());

    // Verify results
    for (&k, &v) in &freq_sequential {
        assert_eq!(freq_mutex.get(&k), Some(&v));
        assert_eq!(freq_rwlock.get(&k), Some(&v));
        assert_eq!(freq_dashmap.get(&k).map(|r| *r), Some(v));
        assert_eq!(freq_rayon_fold.get(&k), Some(&v));
    }

    println!("\nAll results match!");


    // Example: Parallel processing with local state
    println!("\nRunning parallel processing with local state examples...");

    let start = Instant::now();
    let result_sequential = process_data_sequential(&data);
    println!("Sequential processing with {} results completed in {:?}",
             result_sequential.len(), start.elapsed());

    let start = Instant::now();
    let result_threads = process_data_with_threads(&data);
    println!("Standard threading processing with {} results completed in {:?}",
             result_threads.len(), start.elapsed());

    let start = Instant::now();
    let result_rayon = process_data_with_rayon(&data);
    println!("Rayon processing with {} results completed in {:?}",
             result_rayon.len(), start.elapsed());

    let start = Instant::now();
    let result_rwlock = process_data_with_rwlock(&data);
    println!("RwLock processing with {} results completed in {:?}",
             result_rwlock.len(), start.elapsed());

    // Verify results (order might differ but contents should be the same)
    assert_eq!(result_sequential.len(), result_threads.len());
    assert_eq!(result_sequential.len(), result_rayon.len());
    assert_eq!(result_sequential.len(), result_rwlock.len());

    println!("\nAll result counts match!");
    
}

// ===== FREQUENCY MAP EXAMPLES =====

// Sequential frequency map
fn frequency_map_sequential(data: &[i32]) -> HashMap<i32, usize> {
    let mut freq_map = HashMap::new();

    for &x in data {
        *freq_map.entry(x).or_insert(0) += 1;
    }

    freq_map
}

// Mutex-based frequency map with standard threading
fn frequency_map_with_mutex(data: &[i32]) -> HashMap<i32, usize> {

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);


    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let freq_map = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_map = Arc::clone(&freq_map);


        let handle = thread::spawn(move || {
            // Process data in chunks to minimize lock contention
            let mut local_map = HashMap::new();

            for &x in &thread_data[start..end] {
                *local_map.entry(x).or_insert(0) += 1;
            }

            // Merge local map into shared map
            let mut map = thread_map.lock().unwrap();
            for (k, v) in local_map {
                *map.entry(k).or_insert(0) += v;
            }
        });

        handles.push(handle);
    
    }
    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(freq_map)
        .expect("There are still references to the map")
        .into_inner()
        .unwrap()
}

// RwLock-based frequency map with standard threading
fn frequency_map_with_rwlock(data: &[i32]) -> HashMap<i32, usize> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let freq_map = Arc::new(RwLock::new(HashMap::new()));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_map = Arc::clone(&freq_map);        

        let handle = thread::spawn(move || {
            // Create a local map to accumulate counts
            let mut local_map = HashMap::new();

            // Process chunk using local accumulation
            for &x in &thread_data[start..end] {
                *local_map.entry(x).or_insert(0) += 1;
            }

            // After processing the entire chunk, acquire a write lock once
            // and merge all local results
            let mut global_map = thread_map.write().unwrap();
            for (k, v) in local_map {
                *global_map.entry(k).or_insert(0) += v;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(freq_map)
        .expect("There are still references to the map")
        .into_inner()
        .unwrap()
}

// DashMap-based frequency map (concurrent HashMap)
fn frequency_map_with_dashmap(data: &[i32]) -> HashMap<i32, usize> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let freq_map = Arc::new(DashMap::new());
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_map = Arc::clone(&freq_map);

        let handle = thread::spawn(move || {
            for &x in &thread_data[start..end] {
                *thread_map.entry(x).or_insert(0) += 1;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Convert DashMap to HashMap for return
    let mut result = HashMap::new();
    for item in freq_map.iter() {
        result.insert(*item.key(), *item.value());
    }

    result
}

// Rayon fold-based frequency map
fn frequency_map_with_rayon_fold(data: &[i32]) -> HashMap<i32, usize> {
    data.par_iter()
        .fold(
            || HashMap::new(),
            |mut local_map, &x| {
                *local_map.entry(x).or_insert(0) += 1;
                local_map
            }
        )
        .reduce(
            || HashMap::new(),
            |mut acc_map, local_map| {
                for (k, v) in local_map {
                    *acc_map.entry(k).or_insert(0) += v;
                }
                acc_map
            }
        )
}

// ===== PROCESSING WITH LOCAL STATE EXAMPLES =====

// A more complex processing example that generates results
fn complex_process(x: i32) -> Option<(i32, String)> {
    if x % 3 == 0 {
        // Let's generate a result for multiples of 3
        Some((x, format!("Processed {}", x)))
    } else {
        None
    }
}

// Sequential processing
fn process_data_sequential(data: &[i32]) -> Vec<(i32, String)> {
    data.iter()
        .filter_map(|&x| complex_process(x))
        .collect()
}

// Standard threading processing
fn process_data_with_threads(data: &[i32]) -> Vec<(i32, String)> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            // Process locally
            let mut local_results = Vec::new();

            for &x in &thread_data[start..end] {
                if let Some(result) = complex_process(x) {
                    local_results.push(result);
                }
            }

            // Add local results to shared results
            let mut results_vec = thread_results.lock().unwrap();
            results_vec.extend(local_results);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results)
        .expect("There are still references to the results")
        .into_inner()
        .unwrap()
}

// Rayon processing
fn process_data_with_rayon(data: &[i32]) -> Vec<(i32, String)> {
    data.par_iter()
        .filter_map(|&x| complex_process(x))
        .collect()
}

// RwLock-based processing
fn process_data_with_rwlock(data: &[i32]) -> Vec<(i32, String)> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let results = Arc::new(RwLock::new(Vec::new()));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            // Process locally
            let mut local_results = Vec::new();

            for &x in &thread_data[start..end] {
                if let Some(result) = complex_process(x) {
                    local_results.push(result);
                }
            }

            // Add local results to shared results
            let mut results_vec = thread_results.write().unwrap();
            results_vec.extend(local_results);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results)
        .expect("There are still references to the results")
        .into_inner()
        .unwrap()
}