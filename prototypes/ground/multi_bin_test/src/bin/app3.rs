// src/bin/app3.rs
use std::{thread, time};

fn main() {
    println!("App 3 started. Running for 10 seconds...");
    // Example work
     for i in 1..=10 {
        println!("App 3: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 3 finished.");
     // Keep terminal open for a bit on Windows if run directly
    if cfg!(windows) {
         println!("Press Enter to exit App 3...");
         let mut input = String::new();
         std::io::stdin().read_line(&mut input).unwrap();
    }
}