mod note;
mod wave;
mod curves;
mod repeat;
mod line;

use std::env;
use std::io::{BufRead, BufReader, Result};
use wave::Wave;
use std::fs::File;
use crate::curves::sinusoid;
use crate::repeat::{Repeat, REPEAT};


fn io() -> (String, String) {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        (String::from(&args[1]), String::from(&args[2]))
    } else {
        (String::from("input.txt"), String::from("output.wav"))
    }
}

fn main() -> Result<()> {
    let (input, output) = io();

    let amplitude = i16::MAX as f64 / 6.0; // maximum 6 notes at a time
    let fps = 12000;
    let mut wave = Wave::new(File::create(output)?, fps, amplitude, &sinusoid);
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
