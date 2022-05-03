mod note;
mod wave;
mod curves;
mod repeat;
mod line;

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
        _ => ("input.txt".to_string(), "output.txt".to_string()),
    }
}

fn main() -> Result<()> {
    let (input, output) = io();

    let amplitude = i16::MAX as f64 / 6.0; // maximum 6 notes at a time
    let fps = 12000;
    let mut wave = Wave::new(File::create(output)?, fps, amplitude);
    let file = File::open(input)?;
    let reader = BufReader::new(file);
    let mut repeat = Repeat::new();

    wave.start()?;
    reader.lines().map(|line| line.unwrap().trim().to_string())
        .filter(|line| line.len() > 0)
        .for_each(|line| {
            match line {
                _ if line.contains(REPEAT) => line.split_whitespace().for_each(|token| repeat.parse(&mut wave, token)),
                _ if line.chars().next().unwrap().is_ascii_digit() => wave.process(&line, &mut repeat).unwrap(),
                _ => {}
            }
        });
    wave.finish()?;
    Ok(())
}
