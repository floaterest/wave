use std::collections::HashSet;
use crate::line::Line;
use crate::{REPEAT, Wave};
use crate::writer::Writer;

pub struct Repeat {
    pub voltas: Vec<Vec<Line>>,
    pub current: usize,
    pub to_store: HashSet<usize>
}

impl Repeat {
    pub fn new() -> Self {
        Self { voltas: vec![], current: 1, to_store: HashSet::new() }
    }
    // update set of voltas to store
    pub fn start(&mut self, indices: &[usize]) {
        match indices.iter().max() {
            Some(&size) => {
                let size = size + 1;
                if size > self.voltas.len() { self.voltas.resize(size, vec![]) }
                // don't store current volta
                self.to_store = indices.iter().filter(|&&i| i != self.current).cloned().collect();
                let voltas = &mut self.voltas;
                self.to_store.iter().for_each(|&i| voltas[i] = vec![Line::new()]);
            },
            None => panic!("Want to start repeat but indices are empty!")
        }
    }
    /// add a new note to the current voltas
    pub fn push(&mut self, len: usize, freq: f64) {
        for &v in self.to_store.iter() {
            if let Some(line) = self.voltas[v].iter_mut().last() {
                line.notes.push((len, freq));
            }
        }
    }
    /// add new line to each volta to store
    pub fn flush(&mut self, size: usize, offset: usize) {
        for &v in self.to_store.iter() {
            if let Some(line) = self.voltas[v].iter_mut().last() {
                line.size = size;
                line.offset = offset;
            }
            self.voltas[v].push(Line::new());
        }
    }
    /// free all data
    pub fn clear(&mut self) {
        self.to_store.clear();
        self.voltas.clear();
        self.current = 1;
    }
}
