use std::collections::HashSet;
use crate::line::Line;
use crate::Wave;

pub const REPEAT: char = '|';

pub struct Repeat {
    voltas: Vec<Vec<Line>>,
    current: usize,
    to_store: HashSet<usize>
}

impl Repeat {
    pub fn new() -> Self {
        Self { voltas: vec![], current: 1, to_store: HashSet::new() }
    }
    /// clear all data
    fn clear(&mut self) {
        self.to_store.clear();
        self.voltas.clear();
        self.current = 1;
    }
    //#region repeat
    /// update set of volta to record
    fn start(&mut self, indices: &[usize]) {
        let size = *indices.iter().max().unwrap() + 1;
        // if need to reserve space for more voltas
        if size > self.voltas.len() {
            self.voltas.resize(size, vec![]);
        }
        // don't store the current volta
        // ∵ current will never be 0
        // ∴ volta 0 (main repeat) will always be stored
        self.to_store = indices.iter().cloned().filter(|&i| i != self.current).collect();
        let voltas = &mut self.voltas;
        // add empty vec to each new voltas
        self.to_store.iter().for_each(|&i| voltas[i] = vec![Line::new()]);
    }
    /// append repeat to Wave and prepare for the next volta from input
    fn repeat(&mut self, to: &mut Wave) {
        // add repeat to Wave
        self.voltas[0].iter().for_each(|line| line.append(to));
        // ready to store the next volta
        self.current += 1;
        // if the next volta is already stored (∴ won't appear in input)
        if self.voltas.len() > self.current && self.voltas[self.current].len() > 0 {
            // drain the volta to Wave
            self.voltas[self.current].drain(..).for_each(|line| line.append(to));

            self.current += 1;
            // add repeat to Wave again
            self.voltas[0].iter().for_each(|line| line.append(to));
        }
    }
    /// change the current line's size to reserve and offset when flushing for each volta to store
    pub fn resize(&mut self, size: usize, offset: usize) {
        self.to_store.clone().iter().for_each(
            |&v| if let Some(line) = self.voltas[v].iter_mut().last() {
                line.size = size;
                line.offset = offset;
            }
        );
    }
    /// add new line to each volta to store
    pub fn flush(&mut self) {
        let voltas = &mut self.voltas;
        self.to_store.iter().for_each(|&i| voltas[i].push(Line::new()));
    }
    /// push a note to the current line of each volta to store
    pub fn push(&mut self, len: usize, freq: f64) {
        self.to_store.clone().iter().for_each(
            |&v| if let Some(line) = self.voltas[v].iter_mut().last() {
                line.notes.push((len, freq));
            }
        );
    }
    //#endregion repeat
    //#region parse
    fn parse_start(&mut self, token: &str) {
        match token.strip_prefix(REPEAT) {
            Some(":") => self.start(&[0]),
            Some(s) => self.start(
                &s.split('.').filter(|ch| !ch.is_empty())
                    .map(|ch| ch.parse().expect(&format!("Invalid volta token: {}", ch)))
                    .collect::<Vec<usize>>()
            ),
            _ => assert!(false, "Invalid repeat start token: {}", token),
        }
    }
    fn parse_end(&mut self, token: &str, wave: &mut Wave) {
        match token.strip_suffix(REPEAT) {
            // if end of all voltas
            Some("") => self.clear(),
            Some(":") => {
                self.repeat(wave);
                // if doesn't have voltas starting from 1
                if self.voltas.len() == 1 { self.clear(); }
            }
            _ => assert!(false, "Invalid repeat end token: {}", token),
        }
    }
    pub fn parse(&mut self, wave: &mut Wave, token: &str) {
        match token {
            _ if token.ends_with(REPEAT) => self.parse_end(token, wave),
            _ if token.starts_with(REPEAT) => self.parse_start(token),
            _ => assert!(false, "Invalid repeat token: {}", token),
        }
    }
    //#endregion parse
}
