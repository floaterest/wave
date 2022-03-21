mod note;
mod wave;
mod curves;

use std::env;
use std::io::{BufRead, BufReader, Result};
use wave::Wave;
use std::f64::consts::PI;
use std::fs::File;
use crate::curves::sinusoid;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let (input, output) = if args.len() > 2 {
        (String::from(&args[1]), String::from(&args[2]))
    } else {
        (String::from("input.txt"), String::from("output.wav"))
    };

    let amplitude = i16::MAX as f64 / 6.0; // maximum 6 notes at a time
    let fps = 12000;
    let mut w = Wave::new(File::create(output)?, fps, amplitude, &sinusoid);
    let file = File::open(input)?;
    let r = BufReader::new(file);

    w.start()?;
    r.lines().map(|l| l.unwrap())
        .map(|l| String::from(l.trim()))
        .filter(|l| l.len() > 0)
        .filter(|l| l.bytes().next().unwrap().is_ascii_digit())
        .for_each(|l| w.process(&l.split_whitespace().collect::<Vec<&str>>()).unwrap());
    w.finish()?;
    Ok(())
}
