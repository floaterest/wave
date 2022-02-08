mod scanner;

use scanner::Scanner;

fn main() {
    let mut sc = Scanner::default();
    let f: f64 = sc.next();
    let a: f64 = sc.next();
    let d: u32 = sc.next();
    let fps: u32 = sc.next();
    let bits_per_frame: u16 = sc.next();

    println!("frequency: {:?} hz", f);
    println!("amplitude: {:?}", a);
    println!("duration: {:?}s", d);
    println!("frames/sample rate: {:?} fps",fps);
    println!("frame width: {:?} bits",bits_per_frame);
}
