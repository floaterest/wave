mod scanner;
mod wave;

use scanner::Scanner;
use wave::Wave;

fn main() {
    let mut sc = Scanner::default();
    let f: f64 = sc.next();
    let a: f64 = sc.next();
    let d: u32 = sc.next();
    let fps: u32 = sc.next();
    let fname:String=sc.next();

    let w=Wave::new(fps);
    w.write(f,a,d,&fname).unwrap();
}
