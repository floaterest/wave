use std::collections::HashSet;
use crate::Wave;

pub struct Repeat {
    pub voltas: Vec<Vec<Vec<(usize, f64, bool)>>>,
    pub current: usize,
    to_store: HashSet<usize>
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
        self.to_store.iter().for_each(|&i| voltas[i] = vec![vec![]]);
    }
    pub fn flush(&mut self) {
        let voltas = &mut self.voltas;
        self.to_store.iter().for_each(|&i| voltas[i].push(vec![]));
    }
    fn append_repeat(&self, wave: &mut Wave) {
        self.voltas[0].iter().filter(|line| line.len() > 0).for_each(
            |line| {
                line.iter().for_each(|(len, freq, staccato)| wave.append(*len, *freq, *staccato));
                wave.flush(line[0].0).unwrap()
            }
        );
    }
    pub fn repeat(&mut self, wave: &mut Wave) {
        self.append_repeat(wave);
        self.current += 1;
        if self.voltas.len() > self.current && self.voltas[self.current].len() > 0 {
            self.voltas[self.current].drain(..).filter(|line| line.len() > 0).for_each(|line| {
                line.iter().for_each(|(n, freq, staccato)| wave.append(*n, *freq, *staccato));
                wave.flush(line[0].0).unwrap();
            });
            self.current += 1;
            self.append_repeat(wave);
        }
    }
    pub fn push(&mut self, len: usize, freq: f64, staccato: bool) {
        match self.to_store.len() {
            0 => {}
            1 if self.to_store.contains(&self.current) => {}
            _ => {
                let to_store: HashSet<usize> = self.to_store.iter().filter(|&&v| v != self.current).cloned().collect();
                to_store.iter().for_each(
                    |&v| self.voltas[v].iter_mut().last().unwrap().push((len, freq, staccato))
                );
            }
        }
    }
}
