use std::collections::HashMap;
use rayon::prelude::*;
use std::time::{Instant, };

fn is_prime(n: i32) -> bool {
    if n <= 1 || (n > 2 && n % 2 == 0) { return false; }
    (3..=(n as f64).sqrt() as i32).step_by(2).all(|i| n % i != 0)
}

fn time_operation<F, T>(operation_name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("Time for {}: {:?}", operation_name, duration);
    result
}

fn main() {
    // Initialize data
    let data = time_operation("initializing data", || {
        let mut data: Vec<HashMap<String, i32>> = Vec::new();
        for i in 0..10000 {
            let mut map = HashMap::new();
            map.insert("key".to_string(), i);
            data.push(map);
        }
        data
    });

    // Calculate sum
    let sum = time_operation("calculating sum", || {
        data.par_iter()
            .map(|map| *map.get("key").unwrap_or(&0))
            .sum::<i32>()
    });
    println!("Sum: {}", sum);

    // Create num_list (measuring time for the materialization of iterator)
    let num_list = time_operation("creating num_list iterator", || {
        data.par_iter()
            .map(|map| *map.get("key").unwrap_or(&0))
    });

    // Calculate prime count with inspection
    let prime_count = time_operation("calculating prime count with simple filter", || {
        num_list
            .clone()
            .filter(|&x| is_prime(x))
            //.inspect(|x| println!("Found Prime: {}", x))
            .count()
    });
    println!("Number of prime numbers: {}", prime_count);

    // Calculate sum of primes using reduce
    let sum_of_primes = time_operation("calculating sum of primes with reduce", || {
        let result = num_list
            .filter(|&x| is_prime(x))
            .reduce(|| 0, |a, b| a + b);
        result
    });
    println!("Sum of prime numbers: {}", sum_of_primes);

    // Calculate another sum of primes
    let another_sum_of_primes = time_operation("calculating another sum of primes using filter & map", || {
        data.par_iter()
            .map(|map| *map.get("key").unwrap_or(&0))
            .filter(|&x| is_prime(x))
            .sum::<i32>()
    });
    println!("Another sum of prime numbers using filter & map: {}", another_sum_of_primes);

    // Calculate fold sum of primes
    let fold_sum_of_primes = time_operation("calculating fold sum of primes", || {
        data.par_iter()
            .map(|map| *map.get("key").unwrap_or(&0))
            .filter(|&x| is_prime(x))
            .fold(|| 0, |a, b| a + b)
            .sum::<i32>()
    });
    println!("Fold sum of prime numbers: {}", fold_sum_of_primes);
}