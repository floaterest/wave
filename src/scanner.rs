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
            let mut line = self.next_line();
            line.reverse();
            self.buffer = line;
        }
    }
    pub fn next_line(&self) -> Vec<String> {
        loop {
            let mut line = String::new();
            stdin().read_line(&mut line).expect("Scanner: Read line failed");
            line = String::from(line.trim());
            if line.starts_with("//") {
                continue;
            }
            let args: Vec<String> = line.split_ascii_whitespace().map(String::from).collect();
            if args.len() != 0 {
                return args;
            }
        }
    }
}