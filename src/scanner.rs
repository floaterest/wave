use std::str;
use std::io::stdin;

// https://codeforces.com/contest/1168/submission/54903799
#[derive(Default)]
pub struct Scanner {
    buffer: Vec<String>,
}

impl Scanner {
    pub fn next<T: str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.buffer.pop() {
                return token.parse().ok().expect("Scanner: Parse failed");
            }
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Scanner: Read failed");
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}