extern crate core;

mod note;
mod wave;
mod curves;
mod repeat;
mod line;
mod writer;

use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, Result},
};
use wave::Wave;
use repeat::Repeat;

pub const DOTTED: char = '.';
pub const STACCATO: char = '*';
pub const TIE: char = '+';
pub const REPEAT: char = '|';

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
    let mut wave = Wave::new(File::create(output)?, fps, amplitude);
    let reader = BufReader::new(File::open(input)?);
    wave.parse(reader.lines().flatten())?;
    Ok(())
}
