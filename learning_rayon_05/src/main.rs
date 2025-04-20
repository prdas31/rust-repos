
use std::collections::HashMap;
use std::hash::Hash;
use rayon::prelude::*;



enum MyValue {
    Integer(i32),
    Float(f64),
    Text(String),
}

// Add Clone implementation for MyValue
impl Clone for MyValue {
    fn clone(&self) -> Self {
        match self {
            MyValue::Integer(i) => MyValue::Integer(*i),
            MyValue::Float(f) => MyValue::Float(*f),
            MyValue::Text(s) => MyValue::Text(s.clone()),
        }
    }
}



// Generic function that creates a data structure with String keys and enum values
fn create_mixed_data<K, V>() -> Vec<HashMap<K, Vec<V>>>
where
    K: Hash + Eq + From<String>, // Allow conversion from String to K
    V: From<MyValue>, // Allow conversions to V
{
    let mut data: Vec<HashMap<K, Vec<V>>> = Vec::new();

    let mut map1: HashMap<K, Vec<V>> = HashMap::new();

    // Create keys
    let key1: K = "integers".to_string().into();
    let key2: K = "floats".to_string().into();
    let key3: K = "texts".to_string().into();

    // Create vectors with values converted from the Values enum
    let integers: Vec<V> = vec![
        MyValue::Integer(1).into(),
        MyValue::Integer(2).into(),
        MyValue::Integer(3).into(),
    ];

    let floats: Vec<V> = vec![
        MyValue::Float(1.1).into(),
        MyValue::Float(2.2).into(),
        MyValue::Float(3.3).into(),
    ];

    let texts: Vec<V> = vec![
        MyValue::Text("hello".to_string()).into(),
        MyValue::Text("world".to_string()).into(),
    ];

    map1.insert(key1, integers);
    map1.insert(key2, floats);
    map1.insert(key3, texts);

    data.push(map1);

    // Add a second map
    let mut map2: HashMap<K, Vec<V>> = HashMap::new();

    let key4: K = "mixed".to_string().into();
    let mixed: Vec<V> = vec![
        MyValue::Integer(42).into(),
        MyValue::Float(3.14).into(),
        MyValue::Text("Rust".to_string()).into(),
    ];

    map2.insert(key4, mixed);
    data.push(map2);

    data
}

// Generic processing function
fn process_data_generic<K, V, F, R>(
    data: &Vec<HashMap<K, Vec<V>>>,
    value_processor: F,
) -> Vec<R>
where
    K: Hash + Eq + Sync,
    V: Sync + Send,
    R: Send,
    F: Fn(&V) -> R + Sync + Send,
{
    data.par_iter()
        .flat_map(|hashmap| {
            hashmap.par_iter()
                .flat_map(|(_, values)| {
                    values.par_iter()
                        .map(|v| value_processor(v))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()

}

// Helper function to print our data structure
fn print_data(data: &Vec<HashMap<String, Vec<MyValue>>>) {
    for (i, hashmap) in data.iter().enumerate() {
        println!("HashMap #{} contains {} entries:", i + 1, hashmap.len());
        for (key, values) in hashmap {
            println!("  Key '{}' has {} values:", key, values.len());
            for value in values {
                match value {
                    MyValue::Integer(i) => println!("    Integer: {}", i),
                    MyValue::Float(f) => println!("    Float: {}", f),
                    MyValue::Text(s) => println!("    Text: {}", s),
                }
            }
        }
    }
}

fn main() {

    // Create data with String keys and MyValue values
    let mut data: Vec<HashMap<String, Vec<MyValue>>> = create_mixed_data();

    // Now you can work with your populated data structure
    println!("Created {} hashmaps", data.len());

    println!("Original data:");
    print_data(&data);


    // Example 1: Process all values and convert them to strings
    let result1 = process_data_generic(&data, |value| {
        match value {
            MyValue::Integer(i) => i.to_string(),
            MyValue::Float(f) => f.to_string(),
            MyValue::Text(s) => s.clone(),
        }
    });

    println!("\nAll values as strings: {:?}", result1);

    // Example 2: Extract all integer values and double them
    let result2 = process_data_generic(&data, |value| {
        match value {
            MyValue::Integer(i) => Some(i * 2),
            //MyValue::Float(f) => Some(f * 2.0),
            _ => None,
        }
    });

    println!("\nAll integers doubled (with None for non-integers): {:?}", result2);

    // Now let's modify the data structure itself (focusing on map2)
    // Create a mutable copy
    let mut mutable_data = data.clone();

    // Modify the second map
    // Example 3: Modify map2 - transform its values in-place
    if let Some(map2) = mutable_data.get_mut(1) {
        for values in map2.values_mut() {
            for value in values.iter_mut() {
                match value {
                    MyValue::Integer(i) => *i *= 10,
                    MyValue::Float(f) => *f *= 10.0,
                    MyValue::Text(s) => s.push_str(" modified"),
                }
            }
        }
    }

    if mutable_data.len() > 1 {
        // Find and modify the "mixed" entry in the second hashmap
        if let Some(mixed_values) = mutable_data[1].get_mut("mixed") {
            // Transform each value in the vector in place
            for value in mixed_values.iter_mut() {
                match value {
                    MyValue::Integer(i) => *i += 1,
                    MyValue::Float(f) => *f += 1.0,
                    MyValue::Text(s) => s.push_str(" updated"),
                }
            }
        }
    }

    println!("\nAfter modifying map2 in-place:");
    print_data(&mutable_data);
    //
    //


    // Example 4: Add new entries to map2
    if mutable_data.len() > 1 {
        // Add a new key with a new vector
        mutable_data[1].insert(
            "new_entries".to_string(),
            vec![
                MyValue::Integer(100),
                MyValue::Float(99.9),
                MyValue::Text("New value!".to_string()),
            ]
        );
    }

    println!("\nAfter adding new entries to map2:");
    print_data(&mutable_data);


    // Now modify the original data directly

    // Modify all values in map2
    
    /*if let Some(map2) = data.get_mut(1) {
        for values in map2.values_mut() {
            for value in values.iter_mut() {
                match value {
                    MyValue::Integer(i) => *i *= 10,
                    MyValue::Float(f) => *f *= 10.0,
                    MyValue::Text(s) => s.push_str(" modified"),
                }
            }
        }
    }*/

    // Parallel modification of all values in map2
    if let Some(map2) = data.get_mut(1) {
        map2.par_iter_mut()
            .for_each(|(_, values)| {
                values.par_iter_mut().for_each(|value| {
                    match value {
                        MyValue::Integer(i) => *i *= 10,
                        MyValue::Float(f) => *f *= 10.0,
                        MyValue::Text(s) => s.push_str(" modified"),
                    }
                });
            });
    }
    
    println!("\nAfter modifying map2 directly:");    
    print_data(&data);


    // Specifically, modify the "mixed" entry in map2
    if data.len() > 1 {
        if let Some(mixed_values) = data[1].get_mut("mixed") {
            for value in mixed_values.iter_mut() {
                match value {
                    MyValue::Integer(i) => *i += 1,
                    MyValue::Float(f) => *f += 1.0,
                    MyValue::Text(s) => s.push_str(" updated"),
                }
            }
        }
    }

    println!("\nAfter modifying map2 in-place:");
    print_data(&data);

    // Add new entries to map2
    if data.len() > 1 {
        data[1].insert(
            "new_entries".to_string(),
            vec![
                MyValue::Integer(100),
                MyValue::Float(99.9),
                MyValue::Text("New value!".to_string()),
            ]
        );
    }

    println!("\nAfter adding new entries to map2:");
    print_data(&data);

}
