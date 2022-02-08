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
                return token.parse().ok().unwrap();
            }
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}