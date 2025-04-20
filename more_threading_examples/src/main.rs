
use std::sync::Arc;
use rayon::prelude::*;

mod data_structures;
use data_structures::Record;

fn main() {
    // Test setup - create a record with 10 DashMaps
    let mut record = Record::<String, i32>::new(1, 10);
    println!("Created Record with ID: {} and {} DashMaps", record.get_id(), record.get_data_size());

    // Populate the DashMaps with initial data
    for i in 0..10 {
        for j in 0..5 {
            let key = format!("key-{}-{}", i, j);
            let value = i as i32 * 100 + j as i32;
            record.insert_data(i, key, value);
        }
    }

    println!("Initial data inserted. Total keys: {}", record.get_all_keys().len());

    // Create a shared reference for parallel access
    let record_arc = Arc::new(record);

    // Test parallel read operations with rayon
    println!("\nTesting parallel reads with rayon...");
    let results: Vec<Option<i32>> = (0..10).into_par_iter().map(|i| {
        let key = format!("key-{}-{}", i, 0);
        record_arc.get_data_for_key(i, &key)
    }).collect();

    println!("Parallel read results: {:?}", results);

    // Test find_map_with_key in parallel
    println!("\nTesting find_map_with_key in parallel...");
    let finds: Vec<_> = record_arc.get_all_keys().par_iter().take(10).map(|key| {
        let map = record_arc.find_map_with_key(key);
        match map {
            Some(_) => format!("Found map containing key: {}", key),
            None => format!("No map found for key: {}", key),
        }
    }).collect();

    println!("Find results:");
    for result in finds {
        println!("  {}", result);
    }

    // Create a new record for modification tests
    let modifiable_record = Record::<String, i32>::new(2, 5);

    // Wrap the record in an Arc<Mutex<>> for thread-safe mutable access
    let record_mutex = Arc::new(std::sync::Mutex::new(modifiable_record));

    // Populate with test data
    {
        let mut record = record_mutex.lock().unwrap();
        for i in 0..10 {
            let key = format!("mod-key-{}", i);
            record.insert_data(i % 5, key.clone(), i as i32);
            println!("Inserted {} -> {} at index {}", key, i, i % 5);
        }
    } // Release the lock

    println!("\nTesting concurrent modifications with proper synchronization...");

    // Use rayon's scope for parallel execution
    rayon::scope(|s| {
        for i in 0..5 {
            let key = format!("mod-key-{}", i);
            // Clone the Arc for this thread
            let record_ref = Arc::clone(&record_mutex);

            s.spawn(move |_| {
                // Acquire the lock to modify the record
                let mut record = record_ref.lock().unwrap();

                // Perform the modification using the record's method
                if let Some(_map) = record.modify_a_map_with_key(&key, |map| {
                    if let Some(mut entry) = map.get_mut(&key) {
                        *entry += 100;
                        println!("Thread {} updated {} to {}", i, key, *entry);
                    }
                }) {
                    println!("Thread {} successfully modified map for key {}", i, key);
                } else {
                    println!("Thread {} failed to find map with key {}", i, key);
                }
            });
        }
    });


    // Test additional concurrent operations
    println!("\nTesting additional concurrent operations...");

    let record_ref = Arc::clone(&record_mutex);

    // Create a vector to store thread handles
    let mut handles = Vec::new();

    // Spawn threads for additional operations
    for i in 5..10 {
        let key = format!("mod-key-{}", i);
        let record_clone = Arc::clone(&record_ref);

        // Use a standard thread for this part
        let handle = std::thread::spawn(move || {
            let mut record = record_clone.lock().unwrap();

            // Use index i % 5 since we have 5 maps
            let index = i % 5;

            // Get the current value
            let current = record.get_data_for_key(index as u32, &key).unwrap_or(0);

            // Modify it
            record.insert_data(index as u32, key.clone(), current + 200);
            println!("Thread {} updated {} to {}", i, key, current + 200);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Print the final state
    println!("\nFinal state after all concurrent modifications:");
    let record = record_mutex.lock().unwrap();

    // Print all values
    for i in 0..10 {
        let key = format!("mod-key-{}", i);
        let index = i % 5;
        if let Some(value) = record.get_data_for_key(index as u32, &key) {
            println!("  {} = {}", key, value);
        } else {
            println!("  {} = Not found", key);
        }
    }

    // Test get_all_keys
    println!("\nAll keys in modifiable_record:");
    let all_keys = record.get_all_keys();
    for key in all_keys {
        println!("  {}", key);
    }

    println!("\nTests completed!");
}