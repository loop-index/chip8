use crate::chip8::*;
pub mod chip8;

use std::env;

fn main() {
    // Read arguments
    let args: Vec<String> = env::args().collect();

    // Check if there are enough arguments
    if args.len() < 3 {
        println!("Usage: deasm <input> <output>");
        return;
    }

    // Read input file
    let input: Vec<u8> = std::fs::read(&args[1]).expect("Failed to read input file");

    // Assemble input file
    let start_time = std::time::Instant::now();
    let output = disassemble(&input);

    // Write output file
    std::fs::write(&args[2], output).expect("Failed to write output file");

    // Print time taken
    println!("Diassembled file in {}us. Output: {}", start_time.elapsed().as_micros(), &args[2]);
}