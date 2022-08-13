use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

use parsers::input::InputParser;

mod curves;
mod writer;
mod parsers;
mod buffers;

/// read input.txt and/or write to output.wav by default
fn io() -> (String, String) {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => (args[1].to_string(), "output.wav".to_string()),
        3 => (args[1].to_string(), args[2].to_string()),
        _ => ("input.txt".to_string(), "output.wav".to_string()),
    }
}

fn main() -> Result<()> {
    let (input, output) = io();

    let rate = 12000;
    // maximum 6 notes at a time
    let amplitude = i16::MAX as f64 / 6.0;

    let mut parser = InputParser::new(File::create(output)?, amplitude, rate);
    let reader = BufReader::new(File::open(input)?);
    Ok(parser.write(reader.lines().flatten())?)
}
