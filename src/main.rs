// mod note;
mod scanner;
// mod wave;

use scanner::Scanner;
// use wave::Wave;

fn main() {
    let mut sc = Scanner::default();
    let ampl: f64 = sc.next();
    let fps: u32 = sc.next();
    let nbpm: usize = sc.next();
    let fname: String = sc.next();
    dbg!(ampl,fps,fname);
    for _ in 0..nbpm {
        let bpm: u8 = sc.next();
        dbg!(bpm);
        let nlines: usize = sc.next();
        for _ in 0..nlines {
            let mut line = sc.next_line();
            let beat: f32 = line.pop().unwrap().parse().unwrap();
            let notes:Vec<&[u8]> = line.iter().map(|n| n.as_bytes()).collect();
            dbg!(beat,notes);
        }
    }
}
