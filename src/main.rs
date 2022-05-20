use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};

use parsers::Parser;

mod curves;
mod line;
mod writer;
mod parsers;
mod buffers;

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

    let amplitude = i16::MAX as f64 / 6.0; // maximum 6 notes at a time
    let fps = 12000;
    let mut wave = Parser::new(File::create(output)?, fps, amplitude);
    let reader = BufReader::new(File::open(input)?);
    Ok(wave.write(reader.lines().flatten())?)
}
