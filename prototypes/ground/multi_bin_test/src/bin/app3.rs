use std::{thread, time};

fn main() {
    println!("App 3 started. Running for 10 seconds...");
     for i in 1..=10 {
        println!("App 3: Count {}", i);
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("App 3 finished.");
}