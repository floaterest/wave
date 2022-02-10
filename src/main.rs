mod note;
mod scanner;
mod wave;

use std::io::Result;
use scanner::Scanner;
use wave::Wave;

fn main() -> Result<()> {
    let mut sc = Scanner::default();
    let ampl: f64 = sc.next();
    let fps: u32 = sc.next();
    let nbpm: usize = sc.next();
    let fname: String = sc.next();

    let mut w = Wave::new(fps, ampl, &fname);
    w.start()?;
    for _ in 0..nbpm {
        let bpm: f32 = sc.next();
        let nlines: usize = sc.next();
        for _ in 0..nlines {
            let mut line = sc.next_line();
            let beat: f32 = line.pop().unwrap().parse().unwrap();
            let freqs: Vec<f64> = line.iter().map(|n| note::ntof(n.as_bytes())).collect();
            w.write(&freqs, beat * 60.0 / bpm)?;
        }
    }
    w.finish()?;

    Ok(())
}
