use std::thread;
use std::time::Instant;
use std::sync::{Arc, Mutex};

// For Rayon example
use rayon::prelude::*;



fn main() {
    
    let data: Vec<i32> = (0..1_000_000).collect();

    // Example 1: Increment counter for every even number
    println!("Running counter examples...");

    let start = Instant::now();
    let count_sequential = count_evens_sequential(&data);
    println!("Sequential count: {} (took {:?})", count_sequential, start.elapsed());

    let start = Instant::now();
    let count_standard = count_evens_with_threads(&data);
    println!("Standard threading count: {} (took {:?})", count_standard, start.elapsed());

    let start = Instant::now();
    let count_rayon = count_evens_with_rayon(&data);
    println!("Rayon count: {} (took {:?})", count_rayon, start.elapsed());

    // Example 2: Build a histogram
    println!("\nRunning histogram examples...");

    let start = Instant::now();
    let hist_sequential = histogram_sequential(&data);
    println!("Sequential histogram completed in {:?}", start.elapsed());

    let start = Instant::now();
    let hist_standard = histogram_with_threads(&data);
    println!("Standard threading histogram completed in {:?}", start.elapsed());

    let start = Instant::now();
    let hist_rayon = histogram_with_rayon(&data);
    println!("Rayon histogram completed in {:?}", start.elapsed());

    // Verify results
    assert_eq!(count_sequential, count_standard);
    assert_eq!(count_sequential, count_rayon);
    assert_eq!(hist_sequential, hist_standard);
    assert_eq!(hist_sequential, hist_rayon);
    println!("\nAll results match!");
    
    
}

// ===== COUNTER EXAMPLES =====

// Sequential counter
fn count_evens_sequential(data: &[i32]) -> usize {
    data.iter().filter(|&&x| x % 2 == 0).count()
}

// Standard threading counter
fn count_evens_with_threads(data: &[i32]) -> usize {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_counter = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            let local_count = thread_data[start..end].iter()
                .filter(|&&x| x % 2 == 0)
                .count();

            // Update shared counter with local result
            let mut count = thread_counter.lock().unwrap();
            *count += local_count;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let count = *counter.lock().unwrap();
    count
}

fn count_evens_with_rayon(data: &[i32]) -> usize {
    // Solution 1: Direct count using par_iter
    //data.par_iter().filter(|&&x| x % 2 == 0).count()

    // Solution 2: Using shared state (less efficient but for demonstration)
    /*
    let counter = Arc::new(Mutex::new(0));

    data.par_iter().for_each(|&x| {
        if x % 2 == 0 {
            let mut count = counter.lock().unwrap();
            *count += 1;
        }
    });

    let result = *counter.lock().unwrap();
    result*/
    
    // Solution 3: Using par_reduce (more efficient)
    let count = data.par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|_| 1)
        .reduce(|| 0, |a, b| a + b);
    
    count
}

// ===== HISTOGRAM EXAMPLES =====

// We'll use a simple histogram with 10 buckets
type Histogram = [usize; 10];

// Sequential histogram
fn histogram_sequential(data: &[i32]) -> Histogram {
    let mut hist = [0; 10];

    for &x in data {
        
        let bucket = (x % 10) as usize;        
        hist[bucket] += 1;        
    }

    hist
}

// Standard threading histogram
fn histogram_with_threads(data: &[i32]) -> Histogram {
    let num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let chunk_size = (data.len() + num_threads - 1) / num_threads;
    let histogram = Arc::new(Mutex::new([0; 10]));
    let mut handles = vec![];

    for i in 0..num_threads {
        let start = i * chunk_size;
        let end = std::cmp::min(start + chunk_size, data.len());

        if start >= data.len() {
            continue;
        }

        let thread_data = data.to_vec();
        let thread_hist  = Arc::clone(&histogram);

        let handle = thread::spawn(move || {
            // Build a local histogram for this chunk
            let mut local_hist = [0; 10];

            for &x in &thread_data[start..end] {
                let bucket = (x % 10) as usize;
                local_hist[bucket] += 1;
            }

            // Merge local histogram into shared histogram
            let mut hist = thread_hist.lock().unwrap();
            
            for i in 0..10 {
                hist[i] += local_hist[i];
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = *histogram.lock().unwrap();
    result
}

// Rayon histogram
fn histogram_with_rayon(data: &[i32]) -> Histogram {
    // Solution 1: Using mutex (similar to standard threading)
    let histogram = Arc::new(Mutex::new([0; 10]));

    data.par_chunks(1000).for_each(|chunk| {
        // Build a local histogram for this chunk
        let mut local_hist = [0; 10];
        for &x in chunk {
            let bucket = (x % 10) as usize;
            local_hist[bucket] += 1;
        }

        // Merge local histogram into shared histogram
        let mut hist = histogram.lock().unwrap();
        for i in 0..10 {
            hist[i] += local_hist[i];
        }
    });

    let final_hist = *histogram.lock().unwrap();
    final_hist
    
    
    // Solution 2: Using reduction (more idiomatic for Rayon)
    
    /*data.par_iter()
        .fold(
            || [0; 10],
            |mut local_hist, &x| {
                let bucket = (x % 10) as usize;
                local_hist[bucket] += 1;
                local_hist
            }
        )
        .reduce(
            || [0; 10],
            |mut acc, local_hist| {
                for i in 0..10 {
                    acc[i] += local_hist[i];
                }
                acc
            }
        )*/
}