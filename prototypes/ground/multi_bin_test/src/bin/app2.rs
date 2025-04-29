use std::{thread, time};

fn main() {
    println!("App 2 started. Running for 15 seconds...");
     for i in 1..=15 {
        println!("App 2: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 2 finished.");
}