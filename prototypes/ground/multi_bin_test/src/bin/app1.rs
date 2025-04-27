// src/bin/app1.rs
use std::{thread, time};

fn main() {
    println!("App 1 started. Running for 20 seconds...");
    // Example work
    for i in 1..=20 {
        println!("App 1: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 1 finished.");
    // Keep terminal open for a bit on Windows if run directly
    if cfg!(windows) {
         println!("Press Enter to exit App 1...");
         let mut input = String::new();
         std::io::stdin().read_line(&mut input).unwrap();
    }
}