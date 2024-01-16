//
// In a determinisitc build, the output of this program should be the same
// for every execution.
//
// This program generates random numbers, sleeps for 2 seconds,
// and prints to stdout and stderr.
//
// Expected Output:
//
// stdout: json string with the following keys:
//      - Random numbers: rang1, rang2, rang3
//      - UTC time before sleep: utc1
//      - UTC time after sleep: utc2
//      - System time before sleep: system_time1
//      - System time after sleep: system_time2
//      - Elapsed time: system_time1_elapsed
//
//  stderr: "Error: This is an error message"
//
// Contributing Notes:
//  Compile: `cargo build --target wasm32-wasi`
//  Run: `wasmtime run target/wasm32-wasi/debug/deterministic.wasm`
//

// Import rust's io and filesystem module
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use chrono::{Utc, DateTime};

// Entry point to our WASI applications
fn main() {
    // Define a dict to store our output
    let mut dict = HashMap::new();

    // Define random numbers
    let mut rng = rand::thread_rng();
    let n1: u32 = rng.gen();
    let n2: u32 = rng.gen();
    let n3: u32 = rng.gen();

    dict.insert("rang1".to_string(), n1.to_string());
    dict.insert("rang2".to_string(), n2.to_string());
    dict.insert("rang3".to_string(), n3.to_string());

    let utc1 = Utc::now();
    let utc1_str = utc1.format("%+").to_string();
    dict.insert("utc1".to_string(), utc1_str);

    // Define system time, elaspsed time
    let system_time1 = SystemTime::now();

    let date_time1: DateTime<Utc> = system_time1.into();
    let system_time_str = date_time1.format("%+");
    dict.insert("system_time1".to_string(), system_time_str.to_string());

    // we sleep for 2 seconds
    sleep(Duration::new(2, 0));
    match system_time1.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!("{}", elapsed.as_secs());
            dict.insert("system_time1_elapsed".to_string(), elapsed.as_secs().to_string());
        }
        Err(e) => {
            // an error occurred!
            println!("Error: {e:?}");
        }
    }

    // Declare a new UTC after the pause
    let utc2 = Utc::now();
    let utc2_str = utc2.format("%+").to_string();
    dict.insert("utc2".to_string(), utc2_str);

    let json = serde_json::to_string(&dict).unwrap();

    // write to stdout
    println!("{}", json);

    // write to stderr
    eprintln!("Error: {}", "This is an error message");
}
