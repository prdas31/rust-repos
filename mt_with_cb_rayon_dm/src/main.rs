mod data_structures;
mod worker_utils;

use std::sync::Arc;
use crossbeam::channel;
use rayon::prelude::*;
use std::thread;

use data_structures::MyData;

use worker_utils::{
    Operation, create_worker_fn, process_keys_parallel,
    scoped_data_processing, parallel_segment_process, batch_process_parallel
};

fn main() {
    // Initialize Rayon thread pool with a specific number of threads
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    // Create our thread-safe data structure with 8 segments
    let data = Arc::new(MyData::<String, u64>::new(1, 8));

    println!("Data structure ID: {}", data.id());
    println!("Is data structure empty? {}", data.is_empty());

    // Example 1: Direct concurrent access with Rayon
    println!("\nExample 1: Direct concurrent access with Rayon");
    {
        // Create some test data
        (0..1000).into_par_iter().for_each(|i| {
            data.insert(format!("key-{}", i), i as u64);
        });

        println!("Inserted 1000 items");
        println!("Total items: {}", data.len());

        // Process the data concurrently
        let keys: Vec<String> = (0..100).map(|i| format!("key-{}", i)).collect();
        let results = process_keys_parallel(&data, keys, |key, value_opt| {
            match value_opt {
                Some(value) => format!("Found {} with value {}", key, value),
                None => format!("Key {} not found", key),
            }
        });

        for result in results.iter().take(5) {
            println!("{}", result);
        }
        println!("...");

        // Use the for_each method to increment all values
        println!("\nIncrementing all values by 10");
        data.for_each(|_, value| {
            *value += 10;
        });

        // Verify some values were incremented
        let key = "key-5".to_string();
        if let Some(value_ref) = data.get(&key) {
            println!("After increment: key-5 = {}", *value_ref);
        }
    }

    // Example 2: Worker-based approach with Crossbeam channels
    println!("\nExample 2: Worker-based approach with Crossbeam channels");
    {
        // Create channels for communicating with the worker
        let (sender, receiver) = channel::unbounded();

        // Spawn a worker thread
        let worker_data = Arc::clone(&data);
        let worker_handle = thread::spawn(create_worker_fn(worker_data, receiver));

        // Send some operations to the worker
        for i in 1000..1100 {
            let key = format!("key-{}", i);
            sender.send(Operation::Insert(key, i as u64)).unwrap();
        }

        // Get some values back
        for i in 1000..1010 {
            let key = format!("key-{}", i);
            let (response_sender, response_receiver) = channel::bounded(1);
            sender.send(Operation::Get(key, response_sender)).unwrap();

            match response_receiver.recv() {
                Ok(Some(value)) => println!("Retrieved: {}", value),
                Ok(None) => println!("Not found"),
                Err(_) => println!("Error receiving response"),
            }
        }

        // Find entries matching a predicate
        let (response_sender, response_receiver) = channel::bounded(1);
        let predicate = Arc::new(|_: &String, v: &u64| *v > 1095);
        sender.send(Operation::Find(predicate, response_sender)).unwrap();

        match response_receiver.recv() {
            Ok(results) => {
                println!("Found {} entries with value > 1095", results.len());
                for (k, v) in results.iter().take(3) {
                    println!("  {} = {}", k, v);
                }
            },
            Err(_) => println!("Error receiving find results"),
        }

        // Use the Remove operation and verify it worked
        let key_to_remove = "key-1050".to_string();
        sender.send(Operation::Remove(key_to_remove.clone())).unwrap();
        println!("Sent Remove operation for key: {}", key_to_remove);

        // Verify removal worked
        let (response_sender, response_receiver) = channel::bounded(1);
        sender.send(Operation::Get(key_to_remove.clone(), response_sender)).unwrap();
        match response_receiver.recv() {
            Ok(Some(_)) => println!("Error: Key still exists after removal"),
            Ok(None) => println!("Verified key was successfully removed"),
            Err(_) => println!("Error receiving response"),
        }

        // Use Clear operation
        sender.send(Operation::Clear).unwrap();
        println!("Sent Clear operation to worker");

        // Send a couple more operations after Clear to verify it worked
        let key = "post-clear-test".to_string();
        sender.send(Operation::Insert(key.clone(), 42)).unwrap();

        let (response_sender, response_receiver) = channel::bounded(1);
        sender.send(Operation::Get(key.clone(), response_sender)).unwrap();
        match response_receiver.recv() {
            Ok(Some(value)) => println!("After Clear, inserted and retrieved: {}", value),
            _ => println!("Failed to insert/retrieve after Clear"),
        }

        // Shutdown the worker
        sender.send(Operation::Shutdown).unwrap();
        let _ = worker_handle.join();

        // Now repopulate
        println!("Repopulating data after worker...");
        (0..100).into_par_iter().for_each(|i| {
            data.insert(format!("repop-key-{}", i), i as u64);
        });
    }

    // Example 3: Using the transaction method
    println!("\nExample 3: Using transaction method");
    {
        // Insert a key we know exists to ensure the transaction works
        data.insert("transaction-test-key".to_string(), 100);

        let key = "transaction-test-key".to_string();
        let result = data.transaction(&key, |k, v| {
            println!("Transacting on key: {}", k);
            *v *= 2; // Double the value
            *v      // Return the new value
        });

        match result {
            Some(new_value) => println!("New value after transaction: {}", new_value),
            None => println!("Key not found for transaction"),
        }
    }

    // Example 4: Using parallel_segment_process
    println!("\nExample 4: Using parallel_segment_process");
    {
        parallel_segment_process(&data, |segment| {
            // Count values greater than 50 (we repopulated with values 0-99)
            let count = segment.iter().filter(|entry| *entry.value() > 50).count();
            println!("Segment has {} values greater than 50", count);
        });
    }

    // Example 5: Using batch_process_parallel with multiple data structures
    println!("\nExample 5: Using batch_process_parallel");
    {
        // Create additional data structures
        let data_structures: Vec<Arc<MyData<String, u64>>> = (0..3)
            .map(|i| {
                let new_data = Arc::new(MyData::<String, u64>::new(i + 2, 4));
                // Add some data
                for j in 0..10 {
                    new_data.insert(format!("ds{}-key-{}", i, j), (i * 100 + j) as u64);
                }
                new_data
            })
            .collect();

        // Add our original data to the list
        let mut all_data = vec![Arc::clone(&data)];
        all_data.extend(data_structures);

        // Process all data structures in parallel
        batch_process_parallel(&all_data, |ds| {
            println!("Processing data structure {}: {} entries", ds.id(), ds.len());
        });
    }

    // Example 6: Using the keys method
    println!("\nExample 6: Using the keys method");
    {
        // Get all keys for a small sample
        let another_data = MyData::<String, u64>::new(999, 2);
        for i in 0..5 {
            another_data.insert(format!("sample-key-{}", i), i as u64);
        }

        let all_keys = another_data.keys();
        println!("All keys in sample data structure:");
        for key in all_keys {
            println!("  {}", key);
        }
    }

    // Example 7: Scoped threads with Crossbeam
    println!("\nExample 7: Scoped threads with Crossbeam");
    {
        // Use scoped threads to compute statistics for each segment
        let stats = scoped_data_processing(&data, |idx, segment| {
            let count = segment.len();
            let sum: u64 = segment.iter().map(|entry| *entry.value()).sum();
            let avg = if count > 0 { sum as f64 / count as f64 } else { 0.0 };

            println!("Segment {}: {} entries, avg value: {:.2}", idx, count, avg);
            (idx, count, avg)
        });

        println!("Processed {} segments", stats.len());
    }

    // Final statistics
    println!("\nFinal data structure statistics:");
    println!("Total entries: {}", data.len());
    println!("Operation count: {}", data.op_count());

    // Clear the data structure
    println!("\nClearing data structure...");
    data.clear();
    println!("Is data structure empty after clear? {}", data.is_empty());
}