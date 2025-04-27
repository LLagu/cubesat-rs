// src/bin/app2.rs
use std::{thread, time};

fn main() {
    println!("App 2 started. Running for 15 seconds...");
    // Example work
     for i in 1..=15 {
        println!("App 2: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 2 finished.");
     // Keep terminal open for a bit on Windows if run directly
    if cfg!(windows) {
         println!("Press Enter to exit App 2...");
         let mut input = String::new();
         std::io::stdin().read_line(&mut input).unwrap();
    }
}