use std::sync::{Arc, };
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use num_cpus;
use threadpool::ThreadPool;

// Import our data structures module
mod data_structures;
use data_structures::MyDataMaps;


fn initialize_data_structure() -> MyDataMaps<String, i128> {
    println!("Initializing data structure...");

    let mut data_maps = MyDataMaps::new();

    // Add initial values to the first HashMap (index 0)
    // Creating keys with pattern "key_X" where X is 0 to 9
    for i in 0..10 {
        let key = format!("key_{}", i);
        let value = i * 10; // Initial values are multiples of 10

        println!("  Adding initial key-value pair: {} -> {}", key, value);
        data_maps.set_initial_values_for_internal_hash_maps(0, key, value);
    }

    println!("Data structure initialized successfully.\n");
    return data_maps;
}

// Worker function that will be executed by threads in the pool
fn worker_thread(
    thread_id: usize,
    data: Arc<MyDataMaps<String, i128>>,
    keys_to_process: Vec<String>,
    finished: Arc<AtomicBool>
) {
    println!("Worker thread {} started", thread_id);

    // Simulate some processing time
    thread::sleep(Duration::from_millis(100));

    // Process each assigned key
    for key in keys_to_process {
        println!("Thread {} processing key: {}", thread_id, key);

        // Apply different modifications based on thread_id
        match thread_id % 3 {
            0 => {
                // Double the value
                data.modify_value_in_vec_inside_hash_map_with_key(0, &key, |val| {
                    *val *= 2;
                    println!("  Thread {} doubled value for key {}: new value = {}", thread_id, key, *val);
                });
            },
            1 => {
                // Add 5 to the value
                data.modify_value_in_vec_inside_hash_map_with_key(0, &key, |val| {
                    *val += 5;
                    println!("  Thread {} added 5 to value for key {}: new value = {}", thread_id, key, *val);
                });
            },
            _ => {
                // Square the value (with overflow protection)
                data.modify_value_in_vec_inside_hash_map_with_key(0, &key, |val| {
                    // Use checked_mul to avoid panics
                    if let Some(result) = val.checked_mul(*val) {
                        *val = result;
                    } else {
                        // If it overflowed, cap at some maximum value
                        *val = i128::MAX;
                    }
                    println!("  Thread {} squared value for key {}: new value = {}", thread_id, key, *val);
                });
            }
        }

        // Simulate variable processing time
        thread::sleep(Duration::from_millis((50 + thread_id * 10) as u64));
    }

    println!("Worker thread {} completed all tasks", thread_id);

    // If this is the last thread to finish, set the finished flag
    if thread_id == 0 {
        thread::sleep(Duration::from_millis(200)); // Give other threads time to finish
        finished.store(true, Ordering::SeqCst);
    }
}

// Function to print final values
fn print_final_values(data: &MyDataMaps<String, i128>) {
    println!("\nFinal values in data structure:");
    println!("===============================");

    // Create a vector to hold (key, value) pairs for sorted output
    let mut sorted_values = Vec::new();

    // Get the HashMap at index 0
    if let Some(hash_map) = data.get_inner_vector(0) {
        // Iterate through all keys in the first 10 keys
        for i in 0..10 {
            let key = format!("key_{}", i);

            // Try to get the value
            if let Some(arc_mutex) = hash_map.get(&key) {
                if let Ok(guard) = arc_mutex.lock() {
                    sorted_values.push((key.clone(), *guard));
                } else {
                    println!("  Error: Couldn't lock mutex for key {}", key);
                }
            } else {
                println!("  Warning: Key {} not found", key);
            }
        }
    } else {
        println!("  Error: HashMap at index 0 not found");
    }

    // Sort by key
    sorted_values.sort_by(|a, b| a.0.cmp(&b.0));

    // Print in a nice format
    for (key, value) in sorted_values {
        println!("  {} -> {}", key, value);
    }
}

fn main() {


    println!("==== Concurrent Data Structure Manipulation Demo ====\n");

    // Create and initialize the data structure
    let data_maps = initialize_data_structure();

    // Wrap the data structure in Arc for thread-safe sharing
    let shared_data = Arc::new(data_maps);

    // Determine the number of CPU cores and create that many threads
    let num_cores = num_cpus::get();
    println!("Detected {} CPU cores, creating {} worker threads", num_cores, num_cores);

    // Flag to signal when processing is complete
    let finished = Arc::new(AtomicBool::new(false));

    // Create thread handles collection
    //let mut thread_handles = Vec::with_capacity(num_cores);

    // Partition keys among threads
    let keys_per_thread = (10 + num_cores - 1) / num_cores; // Ceiling division

    // Start the worker threads
    println!("\nStarting worker threads...");

    let pool = ThreadPool::new(num_cores);


    for i in 0..num_cores {
        let thread_data = Arc::clone(&shared_data);
        let thread_finished = Arc::clone(&finished);

        // Determine which keys this thread will process
        let start_key = i * keys_per_thread;
        let end_key = std::cmp::min(start_key + keys_per_thread, 10);

        let mut keys = Vec::new();
        for k in 0..10 {
            keys.push(format!("key_{}", k));
        }

        // Spawn the thread
        /*let handle = thread::spawn(move || {
            worker_thread(i, thread_data, keys, thread_finished);
        });

        thread_handles.push(handle);
        */

        // Submit the task to the thread pool
        pool.execute(move || {
            worker_thread(i, thread_data, keys, thread_finished);
        });
    }

    // Wait for all threads to complete
    println!("\nMain thread waiting for all worker threads to complete...");
    /*for (i, handle) in thread_handles.into_iter().enumerate() {
        if let Err(e) = handle.join() {
            println!("Error joining thread {}: {:?}", i, e);
        }
    }*/
    // Wait for all tasks to complete
    pool.join();

    println!("\nAll worker threads have completed their tasks.");

    // Print the final state of the data structure
    print_final_values(&shared_data);

    println!("\n==== Demo Completed Successfully ====");

}
