use std::io::{Write, Read};
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let counter_file = File::open(&args[1]).expect("failed to open counter file");

    let mut counter_str = String::new();
    counter_file.take(100).read_to_string(&mut counter_str)
        .expect("failed to read counter file");
    let mut counter: u32 = counter_str.trim().parse().expect("failed to parse counter");

    counter += 1;

    let mut counter_file = File::create(&args[1]).expect("failed to create counter file");
    write!(counter_file, "{}", counter).expect("failed to write counter file");
}
