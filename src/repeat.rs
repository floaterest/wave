use std::collections::HashSet;
use crate::Wave;

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
