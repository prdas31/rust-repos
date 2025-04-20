mod type_aliases_header;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Add, Deref};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use rand::Rng;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use crate::type_aliases_header::ThreadSafeRW;

// Three different containers demonstrating variance patterns
struct Covariant<T>(PhantomData<T>);        // Covariant in T
struct Contravariant<T>(PhantomData<fn(T)>); // Contravariant in T
struct Invariant<T>(PhantomData<fn(T) -> T>); // Invariant in T

fn example_of_variant_use_with_phantom_data<'a>() {

    // Let's create a lifetime hierarchy
    let long_lived_str = "hello";            // 'static lifetime

    // Inside this scope, we have a shorter lifetime 'a
    {
        let short_lived_str: &str = "world"; // shorter lifetime 'a

        // Covariance example: &'static str is a subtype of &'a str
        // So Covariant<&'static str> is a subtype of Covariant<&'a str>
        let _covariant_static: Covariant<&'static str> = Covariant(PhantomData);
        let _covariant_a: Covariant<&'a str> = _covariant_static; // This works!

        // Contravariant example: &'a str is a subtype of &'static str
        // Contravariance example: with function parameters, subtyping is reversed
        // A function taking &'a str can accept a &'static str
        // So Contravariant<&'a str> is a subtype of Contravariant<&'static str>
        let _contra_a: Contravariant<&'a str> = Contravariant(PhantomData);
        let _contra_static: Contravariant<&'static str> = _contra_a; // This works!

        // Invariant example: &'a str is a subtype of &'a str
        // So Invariant<&'a str> is a subtype of Invariant<&'a str>
        let _invariant_a: Invariant<&'a str> = Invariant(PhantomData);
        let _invariant_b: Invariant<&'a str> = _invariant_a; // This works!

        // Invariance example: no subtyping relationships at all
        let _invariant_static: Invariant<&'static str> = Invariant(PhantomData);
        // let _invariant_a: Invariant<&'a str> = _invariant_static; // ERROR! No conversion allowed

    }

}

trait PersonTrait: Any {
    fn name(&self) -> &str; // Optional: override default implementation
    fn greet(&self) {
        println!("Hello, my name is {}", self.name());
    }
}

struct Person { name: String }

impl PersonTrait for Person {
    fn name(&self) -> &str {
        &self.name
    }
    // Optional: override default implementation
    fn greet(&self) {
        println!("Hello, my name is {}", self.name());
    }
}


struct Student { name: String, grade: u8 }

impl PersonTrait for Student {
    fn name(&self) -> &str {
        &self.name
    }

    // Optional: override default implementation
    fn greet(&self) {
        println!("Hello, I'm a student named {} in grade {}", self.name(), self.grade);
    }
}

fn process_names(people: &mut Vec< Person>) {
    // Can add any Person, including non-Students
    people.push(Person { name: "Alice".to_string() });

    for person in people {
        println!("Name: {}", person.name());
    }
}

type TsHm<K, V> = HashMap<K, Arc<RwLock<V>>>;
type TsVhm<K, V> = Vec<TsHm<K, V>>;


fn main() {
    println!("Hello, world!");
    example_of_variant_use_with_phantom_data();

    let mut students: Vec<Student> = vec![
        Student { name: "Bob".to_string(), grade: 10 },
        Student { name: "Tim".to_string(), grade: 9 }
    ];

    //let _= students.iter().map(|s| s.greet()).collect::<Vec<_>>(); //

    // Most idiomatic when you just want the side effects
    students.iter().for_each(|s| s.greet());

    // This would NOT compile in Rust:
    // process_names(&mut students);

    // Instead, we would need to explicitly convert:

    let mut people = students.iter().map(|s| Person {name: s.name.clone() }).collect::<Vec<Person>>();

    process_names(&mut people);

    for person in people {
        //person.greet();
        print!("My name is: {}", person.name());

        if let Some(matching_student) = students.iter().find(|s| s.name == person.name()) {
            println!(", Grade: {}", matching_student.grade);
        } else {
            println!(", and I am not a student anymore in schools");
        }
    }

    // Create a vector of thread-safe HashMaps
    //let mut data: TsVhm<String, i32> = Vec::new();

    let mut data = Vec::<TsHm<String, i32>>::new(); //Turbofish syntax


    // Initialize with two HashMaps, each with three entries
    data.push([
        (String::from("apple"), Arc::new(RwLock::new(10))),
        (String::from("banana"), Arc::new(RwLock::new(20))),
        (String::from("cherry"), Arc::new(RwLock::new(30)))
    ].into_iter().collect()); //collect::<TsHm<String, i32>>()

    data.push([
        (String::from("dog"), Arc::new(RwLock::new(100))),
        (String::from("elephant"), Arc::new(RwLock::new(200))),
        (String::from("fox"), Arc::new(RwLock::new(300)))
    ].into_iter().collect());

    // Print initial state
    println!("\n\nInitial state:");
    print_data(&data);

    println!("\nNow starting the Threading Operations...\n");
    // Wrap in Arc for sharing across threads
    let shared_data = Arc::new(data);

    // Create a vector to hold our thread handles
    let mut handles = vec![];

    // Number of threads to spawn
    // Number of threads to spawn - use available cores
    let num_threads = thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1); // Fallback to 1 if we can't determine

    println!("Spawning {} threads (based on available cores)\n", num_threads);

    // Ready to create and launch the threads...

    // Create multiple threads
    for thread_id in 0..num_threads {

        // Clone the Arc for this thread
        let thread_data = Arc::clone(&shared_data);

        // Spawn a thread
        let handle = thread::spawn(move || {

            let mut rng = rand::rngs::ThreadRng::default();

            // Let Each thread to perform 10 operations
            for op_num in 0..10 {

                // Randomly choose to read or write (70% read, 30% write)
                let is_write = rng.random_ratio(3, 10);

                // Choose a random HashMap and key
                let map_idx = rng.random_range(0..thread_data.len());
                let map = &thread_data[map_idx];

                // Get all keys for the chosen map
                let keys: Vec<&String> = map.keys().collect();
                if keys.is_empty() {
                    continue;
                }

                let key_idx = rng.random_range(0..keys.len());
                let key = keys[key_idx];

                // Try to access the value
                if let Some(value_lock) = map.get(key) {

                    if is_write {
                        // Write operation - generate a new random value

                        match value_lock.write() {

                            Ok(mut value) => {
                                let new_value = rng.random_range(1..1000);
                                let old_value = *value;
                                *value = new_value;

                                println!(
                                    "Thread {}: WRITE - Map[{}], Key '{}': old value {} -> new value {}",
                                    thread_id, map_idx, key, old_value, new_value
                                );

                                // Sleep a bit to simulate work
                                thread::sleep(Duration::from_millis(rng.random_range(50..150)));
                            },

                            Err(e) => {
                                println!("Thread {}: Write lock error: {}", thread_id, e);
                            }
                        }
                    } else {
                        // Read operation
                        match value_lock.read() {
                            Ok(value) => {
                                println!(
                                    "Thread {}: READ  - Map[{}], Key '{}': current value {}",
                                    thread_id, map_idx, key, *value
                                );

                                // Sleep a bit to simulate work
                                thread::sleep(Duration::from_millis(rng.random_range(20..100)));
                            },
                            Err(e) => {
                                println!("Thread {}: Read lock error: {}", thread_id, e);
                            }
                        }
                    }
                }

                // Add some randomness to timing
                thread::sleep(Duration::from_millis(rng.random_range(10..50)));
            }

            println!("Thread {} completed all operations", thread_id);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Print final state
    println!("\nFinal state after all threads completed:");
    print_data(&shared_data);

    print_hm_contents(&shared_data[0]);
}

fn print_data<K, V>(data: &TsVhm<K, V>)
where
    K: std::fmt::Display,
    V: std::fmt::Display
{

    for (map_idx, map) in data.iter().enumerate() {

        println!("Map {}:", map_idx);

        for (key, value_lock) in map {

            match value_lock.read() {

                Ok(value) => {
                    println!("  {}: {}", key, *value);
                },
                Err(e) => {
                    println!("Key Error {}: <error: {}>", key, e);
                }
            }
        }
    }

    println!();

}

fn print_hm_contents<K, V>(map: &TsHm<K, V>)
where
    K: Display + Eq,
    V: Display + Copy + Into<i32> // std::ops::Add<Output = V> + From<i32> // V must be convertible to i32
{
    println!("Map Contents are as follows:");

    let mut total_value = 0i32; //V::from(0);  // Initialize with the correct type

    for (k, v_lock) in map {

        match v_lock.read() {

            Ok(v) => {
                println!("Key: {} contains Value: {}", k, v);

                total_value = total_value + (*v).into(); //+ *v;  // Use the guard you already have
            },
            Err(e) => {
                println!("Key Error {}: <error: {}>", k, e);
            }
        }
    }

    println!("Total value: {}", total_value);
}