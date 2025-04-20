mod thread_safe_map;

use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use thread_safe_map::ThreadSafeMap;  // Import the struct from the module

fn main() {
    println!("Hello, world!");


    // Arc<RwLock<T>> pattern: Multiple readers OR single writer
    let counter = Arc::new(RwLock::new(0));

    // Arc<Mutex<T>> pattern: Single access at a time (readers and writers)
    let log = Arc::new(Mutex::new(Vec::<String>::new()));

    let mut handles = vec![];


    // Spawn 5 threads
    for id in 0..5 {
        let counter_clone = Arc::clone(&counter);
        let log_clone = Arc::clone(&log);


        let handle = thread::spawn(move || {
            // Read the counter (multiple readers can access simultaneously)
            {
                let counter_value = counter_clone.read().unwrap();
                println!("Thread {} read counter value: {}", id, *counter_value);

                // Log this read operation (exclusive access)
                let mut log_guard = log_clone.lock().unwrap();
                log_guard.push(format!("Thread {} read the counter value: {}", id, *counter_value));
            } // read guard is dropped here

            // Write to the counter (exclusive access)
            {
                let mut counter_value = counter_clone.write().unwrap();
                *counter_value += 1;
                println!("Thread {} incremented to: {}", id, *counter_value);

                // Log this write operation (exclusive access)
                let mut log_guard = log_clone.lock().unwrap();
                log_guard.push(format!("Thread {} modified counter value: {}", id, *counter_value));
            } // write guard is dropped here

        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Print the final value and log
    println!("Final counter value: {}", *counter.read().unwrap());
    println!("Operation log:");
    for entry in log.lock().unwrap().iter() {
        println!("Log Entries:  {}", entry);
    }

    // Shared data structures
    let data = Arc::new(RwLock::new(vec![0; 5]));
    let result_sum = Arc::new(Mutex::new(0));

    // Create a barrier that waits for 5 threads
    let barrier = Arc::new(Barrier::new(5));

    let mut handles = vec![];

    // Spawn 5 threads
    for id in 0..5 {
        let data_clone = Arc::clone(&data);
        let result_clone = Arc::clone(&result_sum);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            println!("Thread {} starting phase 1 - initialization", id);

            // Phase 1: Each thread updates its own position in the vector
            {
                let mut data_guard = data_clone.write().unwrap();
                data_guard[id] = (id + 1) * 10;
                println!("Thread {} initialized position {} with value {}",
                         id, id, data_guard[id]);

                // Simulate some work
                thread::sleep(Duration::from_millis(100 * (id as u64 + 1)));
            }

            // Wait for all threads to complete phase 1
            println!("Thread {} waiting at barrier after phase 1", id);
            barrier_clone.wait();
            println!("Thread {} continuing to phase 2", id);

            // Phase 2: Each thread reads the entire vector and adds to the sum
            {
                let data_guard = data_clone.read().unwrap();
                let sum: usize  = data_guard.iter(). sum();
                println!("Thread {} calculated sum: {}", id, sum);

                let mut result_guard = result_clone.lock().unwrap();
                *result_guard += sum;
            }

            // Wait for all threads to complete phase 2
            barrier_clone.wait();
            println!("Thread {} completed all phases", id);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Print the final result
    println!("Final vector: {:?}", *data.read().unwrap());
    println!("Total sum (from all 5 threads): {}", *result_sum.lock().unwrap());
    // We expect the sum to be 150 * 5 = 750 because each thread adds the same sum


    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn 5 threads
    for id in 0..5 {
        let counter_clone = Arc::clone(&atomic_counter);
        let handle = thread::spawn(move || {

            // Custom modification function
            let modify_fn = |x| x * 2 + 1;

            counter_clone.fetch_add(1, Ordering::SeqCst);

            // Implementation using compare_exchange
            let mut current = counter_clone.load(Ordering::Relaxed);

            loop {

                let new_value = modify_fn(current);

                match counter_clone.compare_exchange(current, new_value, Ordering::SeqCst, Ordering::Relaxed) {

                    Ok(_) => break, // Success!
                    Err(actual) => current = actual, // Try again with the updated value
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final atomic counter value: {}", atomic_counter.load(Ordering::SeqCst));


    // Create a shared map
    let map = ThreadSafeMap::new();

    // Insert initial values
    map.insert("counter".to_string(), 0);

    // Create handles for 5 threads
    let mut handles = vec![];

    // Spawn 5 threads
    for id in 0..5 {
        let map_clone = map.clone(); // Clone the wrapper, not the inner data

        let handle = thread::spawn(move || {
            // Each thread reads the current value
            if let Some(current) = map_clone.get(&"counter".to_string()) {
                println!("Thread {} read value: {}", id, current);

                // Each thread updates the value
                let new_value = current + 1;
                map_clone.insert("counter".to_string(), new_value);
                println!("Thread {} updated value to: {}", id, new_value);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Print the final value
    println!("Final value: {:?}", map.get(&"counter".to_string()));
}
