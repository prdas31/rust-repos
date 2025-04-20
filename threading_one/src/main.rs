use std::thread;
use std::time::Duration;

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

fn main() {

    // Use an AtomicI32 wrapped in an Arc
    let a = Arc::new(AtomicI32::new(5));

    // Clone the Arc for the child thread
    let a_child = Arc::clone(&a);

    // create a thread
    let handle = thread::spawn(

    move || {

        // this closure will run in the new thread - everything in here runs in its own separate thread

        for i in 1..10 {
            println!("Child thread {} with ID: {:?}", a_child.fetch_add(i, Ordering::Relaxed) + i, thread::current().id());
            thread::sleep(Duration::from_millis(100));
        }
    });

    // main thread continues to run
    for i in 1..5 {
        println!("Main thread: {}", i);
        thread::sleep(Duration::from_millis(200));
    }

    // wait for the spawned thread to finish
    handle.join().unwrap();

    println!("a is now {}", a.load(Ordering::Acquire));
}
