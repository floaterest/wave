use std::collections::HashSet;
use crate::Wave;

pub const REPEAT: char = '|';

#[derive(Clone)]
pub struct Line {
    size: usize,
    offset: usize,
    notes: Vec<(usize, f64)>
}

impl Line {
    pub fn new() -> Self {
        Self {
            size: 0,
            offset: 0,
            notes: vec![]
        }
    }
}

pub struct Repeat {
    pub voltas: Vec<Vec<Line>>,
    pub current: usize,
    to_store: HashSet<usize>
}

fn append_line(line: &Line, wave: &mut Wave) {
    if line.size == 0 { return; }
    wave.resize(line.size);
    line.notes.iter().for_each(|(n, freq)| wave.append(*n, *freq));
    wave.flush(line.offset).unwrap();
}

impl Repeat {
    pub fn new() -> Self {
        Self { voltas: vec![], current: 1, to_store: HashSet::new() }
    }
    pub fn clear(&mut self) {
        self.to_store.clear();
        self.voltas.clear();
    }
    pub fn start(&mut self, indices: &[usize]) {
        let size = *indices.iter().max().unwrap() + 1;
        if size > self.voltas.len() {
            self.voltas.resize(size, vec![]);
        }
        self.to_store = indices.iter().cloned().filter(|&i| i != self.current).collect();
        let voltas = &mut self.voltas;
        self.to_store.iter().for_each(|&i| voltas[i] = vec![Line::new()]);
    }
    pub fn flush(&mut self) {
        let voltas = &mut self.voltas;
        self.to_store.iter().for_each(|&i| voltas[i].push(Line::new()));
    }
    fn append_repeat(&self, wave: &mut Wave) {
        self.voltas[0].iter().for_each(|line| append_line(line, wave));
    }
    pub fn repeat(&mut self, wave: &mut Wave) {
        self.append_repeat(wave);
        self.current += 1;
        if self.voltas.len() > self.current && self.voltas[self.current].len() > 0 {
            self.voltas[self.current].drain(..).for_each(|line| append_line(&line, wave));
            self.current += 1;
            self.append_repeat(wave);
        }
    }
    pub fn set(&mut self, size: usize, offset: usize) {
        match self.to_store.len() {
            0 => {}
            1 if self.to_store.contains(&self.current) => {}
            _ => {
                let to_store: HashSet<usize> = self.to_store.iter().filter(|&&v| v != self.current).cloned().collect();
                to_store.iter().for_each(
                    |&v| {
                        self.voltas[v].iter_mut().last().unwrap().offset = offset;
                        self.voltas[v].iter_mut().last().unwrap().size = size;
                    }
                );
            }
        }
    }
    pub fn push(&mut self, len: usize, freq: f64) {
        match self.to_store.len() {
            0 => {}
            1 if self.to_store.contains(&self.current) => {}
            _ => {
                let to_store: HashSet<usize> = self.to_store.iter().filter(|&&v| v != self.current).cloned().collect();
                to_store.iter().for_each(
                    |&v| self.voltas[v].iter_mut().last().unwrap().notes.push((len, freq))
                );
            }
        }
    }
}

fn parse_repeat_end(repeat: &mut Repeat, wave: &mut Wave, token: &str) {
    match token.strip_suffix(REPEAT) {
        // if end of all voltas
        Some("") => repeat.clear(),
        Some(":") => {
            repeat.repeat(wave);
            // if doesn't have voltas starting from 1
            if repeat.voltas.len() == 1 { repeat.clear(); }
        }
        _ => assert!(false, "Invalid repeat end token: {}", token),
    }
}

fn parse_repeat_start(repeat: &mut Repeat, token: &str) {
    match token.strip_prefix(REPEAT) {
        Some(":") => repeat.start(&[0]),
        Some(s) => repeat.start(
            &s.split('.').filter(|ch| !ch.is_empty())
                .map(|ch| ch.parse().expect(&format!("Invalid volta token: {}", ch)))
                .collect::<Vec<usize>>()
        ),
        _ => assert!(false, "Invalid repeat start token: {}", token),
    }
}

pub fn parse_repeat(repeat: &mut Repeat, wave: &mut Wave, token: &str) {
    match token {
        _ if token.ends_with(REPEAT) => parse_repeat_end(repeat, wave, token),
        _ if token.starts_with(REPEAT) => parse_repeat_start(repeat, token),
        _ => assert!(false, "Invalid repeat token: {}", token),
    }
}
