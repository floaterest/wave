use std::slice::Iter;

pub const DOTTED: char = '.';
pub const STACCATO: char = '*';
pub const TIE: char = '+';

#[derive(Clone, Debug)]
pub struct Chord {
    pub size: usize,
    pub length: usize,
    pub frequencies: Vec<f64>,
}

impl Chord {
    pub fn new() -> Self {
        Self { size: 0, length: 0, frequencies: Vec::new() }
    }
    /// return number of beats
    pub fn parse_length(token: &str) -> f64 {
        let is_tie = || token.chars().all(|ch| ch.is_ascii_digit() || ch == TIE);
        let strip = |suffix: char, scale: f64| {
            scale / token.strip_suffix(suffix).unwrap().parse::<f64>().unwrap()
        };

        match token.parse::<usize>() {
            Ok(length) => 1.0 / length as f64,
            Err(..) => match token.chars().last() {
                Some(DOTTED) => strip(DOTTED, 1.5),
                Some(STACCATO) => strip(STACCATO, 0.5),
                _ if is_tie() => token.split(TIE).flat_map(|s| s.parse::<f64>()).map(|f| 1.0 / f).sum(),
                _ => panic!("Invalid token as node length: {}", token),
            }
        }
    }
    /// scale all frequencies
    // pub fn scale(&mut self, scale: f64) {
    //     self.frequencies.iter_mut().for_each(|f| *f *= scale);
    // }
    pub fn push(&mut self, frequency: f64) {
        self.frequencies.push(frequency);
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0 || self.length == 0 || self.frequencies.is_empty()
    }
    pub fn drain(&mut self) -> Self {
        let copy = self.clone();
        (self.size, self.length) = (0, 0);
        self.frequencies.clear();
        copy
    }
}

#[derive(Clone, Debug)]
pub struct Line {
    chords: Vec<Chord>
}

impl Line {
    pub fn new() -> Self {
        Self { chords: Vec::new() }
    }
    pub fn offset(&self) -> usize {
        self.chords.iter().map(|ch| ch.length).min().unwrap()
    }
    pub fn size(&self) -> usize {
        self.chords.iter().map(|ch| ch.size).max().unwrap()
    }
    pub fn push(&mut self, chord: Chord) {
        self.chords.push(chord);
    }
    pub fn chords(&self) -> Iter<Chord> {
        self.chords.iter()
    }
}