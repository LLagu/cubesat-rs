use std::{thread, time};

fn main() {
    println!("App 1 started. Running for 20 seconds...");
    for i in 1..=20 {
        println!("App 1: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 1 finished.");
}