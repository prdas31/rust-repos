// Example 1: Basic Parallel Iteration

use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex};

// For Rayon example
use rayon::prelude::*;


fn main() {

    // Create a large vector for our examples
    let data: Vec<i32> = (0..1_000_000).collect();

    println!("Running sequential sum...");
    let start = Instant::now();
    let sum_sequential = sum_sequential(&data);
    println!("Sequential sum: {} (took {:?})", sum_sequential, start.elapsed());

    println!("\nRunning sum with Rayon...");
    let start = Instant::now();
    let sum_rayon = sum_with_rayon(&data);
    println!("Rayon sum: {} (took {:?})", sum_rayon, start.elapsed());

    println!("\nRunning sum with standard threading...");
    let start = Instant::now();
    let sum_threads = sum_with_threads(&data);
    println!("Threads sum: {} (took {:?})", sum_threads, start.elapsed());


    println!("\n\nRunning sequential map...");
    let start = Instant::now();
    let result_sequential = map_sequential(&data);
    println!("Sequential map completed in {:?}", start.elapsed());

    println!("\nRunning map with Rayon...");
    let start = Instant::now();
    let result_rayon = map_with_rayon(&data);
    println!("Rayon map completed in {:?}", start.elapsed());


    println!("\nRunning map with standard threading...");
    let start = Instant::now();
    let result_threads = map_with_threads(&data);
    println!("Threads map completed in {:?}", start.elapsed());

    // Verify results are the same
    assert_eq!(result_sequential[0..10], result_rayon[0..10]);
    assert_eq!(result_sequential[0..10], result_threads[0..10]);
    println!("\nAll results match!");


    println!("\n\nRunning sequential filter...");
    let start = Instant::now();
    let result_sequential = filter_sequential(&data);
    println!("Sequential filter found {} elements in {:?}",
             result_sequential.len(), start.elapsed());

    println!("\nRunning filter with Rayon...");
    let start = Instant::now();
    let result_rayon = filter_with_rayon(&data);
    println!("Rayon filter found {} elements in {:?}",
             result_rayon.len(), start.elapsed());

    println!("\nRunning filter with standard threading...");
    let start = Instant::now();
    let result_threads = filter_with_threads(&data);
    println!("Threads filter found {} elements in {:?}",
             result_threads.len(), start.elapsed());

    // Verify results
    assert_eq!(result_sequential.len(), result_rayon.len());
    assert_eq!(result_sequential.len(), result_threads.len());
    println!("\nAll result lengths match!");


}

// Sequential version
fn sum_sequential(data: &[i32]) -> i128 {
    data.iter().map(|&x| x as i128 ).sum()
}

// Rayon parallel version
fn sum_with_rayon(data: &[i32]) -> i128 {
    data.par_iter().map(|&x| x as i128 ).sum()
}

// Standard threading version
fn sum_with_threads(data: &[i32]) -> i128 {
    // Determine number of threads (same as Rayon would use by default)
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    println!("\nTotal number of threads to be used for this operation: {}\n", num_threads);

    // Calculate chunk size for each thread
    let chunk_size = (data.len() + num_threads - 1) / num_threads;

    // Shared result that threads will update
    let result = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    // Create threads to process chunks
    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        // Skip empty chunks
        if start >= data.len() {
            continue;
        }

        // Clone Arc for this thread
        let thread_data = data.to_vec();
        let thread_result = Arc::clone(&result);

        // Spawn thread
        let handle = thread::spawn(move || {
            let chunk_sum: i128 = thread_data[start..end].iter().map(|&x| x as i128).sum();

            // Update shared result
            let mut sum = thread_result.lock().unwrap();
            *sum += chunk_sum;
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Return final result - properly retrieving the value
    let final_result = *result.lock().unwrap();
    final_result

}

// A slightly expensive operation to demonstrate parallelism benefits
fn expensive_calculation(x: i32) -> i128 {

    // Simulate some computational work
    let mut result = x as i128;

    for _ in 0..100 {
        result = (result * result) % 997; // Arbitrary calculation
    }
    result
}

// Sequential version
fn map_sequential(data: &[i32]) -> Vec<i128> {
    data.iter()
        .map(|&x| expensive_calculation(x))
        .collect()
}

// Rayon parallel version
fn map_with_rayon(data: &[i32]) -> Vec<i128> {
    data.par_iter()
        .map(|&x| expensive_calculation(x))
        .collect()
}

// Standard threading version
fn map_with_threads(data: &[i32]) -> Vec<i128> {

    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let mut result = vec![0; data.len()];

    let mut handles = vec![];

    // Create threads to process chunks
    for i in 0..num_threads {


        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        // Skip empty chunks
        if start >= data.len() {
            continue;
        }

        // Clone data for this thread
        let thread_data = data.to_vec();

        // Spawn thread
        let handle = thread::spawn(move || {
            // Process this chunk and return results
            let mut chunk_result = Vec::with_capacity(end - start);
            for j in start..end {
                chunk_result.push(expensive_calculation(thread_data[j]));
            }
            (start, chunk_result)
        });

        handles.push(handle);
    }


    // Collect results from all threads
    for handle in handles {
        let (start, chunk_result) = handle.join().unwrap();
        // Copy results to the right positions in the final result vector
        for (i, &val) in chunk_result.iter().enumerate() {
            result[start + i] = val;
        }
    }

    result

}

// A slightly complex predicate to benefit from parallelism
fn is_prime(n: i32) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }

    true
}

// Sequential version
fn filter_sequential(data: &[i32]) -> Vec<i32> {
    data.iter()
        .filter(|&&x| is_prime(x))
        .cloned()
        .collect()
}

fn filter_with_rayon(data: &[i32]) -> Vec<i32> {
    data.par_iter()
        .filter(|&&x| is_prime(x))
        .cloned()
        .collect()
}

// Standard threading version
fn filter_with_threads(data: &[i32]) -> Vec<i32> {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let result = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Create threads to process chunks
    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        // Skip empty chunks
        if start >= data.len() {
            continue;
        }

        // Clone data for this thread
        let thread_data = data.to_vec();
        let thread_result = Arc::clone(&result);

        // Spawn thread
        let handle = thread::spawn(move || {
            // Filter elements in this chunk
            let mut chunk_result = Vec::new();
            for j in start..end {
                if is_prime(thread_data[j]) {
                    chunk_result.push(thread_data[j]);
                }
            }

            // Add results to shared vector
            let mut result_vec = thread_result.lock().unwrap();
            result_vec.extend(chunk_result);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Return final result
    Arc::try_unwrap(result)
        .expect("There are still references to the result")
        .into_inner()
        .unwrap()
}