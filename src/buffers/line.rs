use std::cell::RefCell;
use std::rc::Rc;
use std::slice::Iter;

const TIE: char = '+';
const DOTTED: char = '.';
const STACCATO: char = '*';

//#region Chord
#[derive(Clone, Debug)]
pub struct Chord {
    /// length of the notes
    pub length: usize,
    /// frequency of each note
    pub frequencies: Vec<f64>,
    // made this field only for you, staccato
    /// duration that the notes occupies
    pub size: usize,
}

impl Chord {
    /// return number of beats
    pub fn parse_length(token: &str) -> f64 {
        // check if token is a tie
        let is_tie = || token.chars().all(|ch| ch.is_ascii_digit() || ch == TIE);
        // strip the suffix and convert integer to number of beats
        let strip = |suffix: char, scale: f64| {
            scale / token.strip_suffix(suffix).unwrap().parse::<f64>().unwrap()
        };

        match token.parse::<usize>() {
            // normal note value
            Ok(length) => 1.0 / length as f64,
            Err(..) => match token.chars().last() {
                Some(DOTTED) => strip(DOTTED, 1.5),
                Some(STACCATO) => strip(STACCATO, 0.5),
                // sum up each value
                _ if is_tie() => token.split(TIE).flat_map(|s| s.parse::<f64>()).map(|f| 1.0 / f).sum(),
                _ => panic!("Invalid token as node length: {}", token),
            }
        }
    }
    /// scale all frequencies
    // pub fn scale(&mut self, scale: f64) {
    //     self.frequencies.iter_mut().for_each(|f| *f *= scale);
    // }
    /// push a new frequency to chord
    pub fn push(&mut self, frequency: f64) {
        self.frequencies.push(frequency);
    }
    /// extend a chord to self
    pub fn extend(&mut self, rhs: &Chord) {
        if self.frequencies.is_empty() {
            // clone rhs to self
            *self = rhs.clone()
        } else {
            assert_eq!(self.length, rhs.length, "attempt to add a chord without equal length");
            assert_eq!(self.size, rhs.size, "attempt to add a chord without equal size");
            self.frequencies.extend(rhs.frequencies.iter());
        }
    }
}
//#endregion  Chord

//#region Line
/// collections of chords to be played at the same time
#[derive(Clone, Debug)]
pub struct Line {
    chords: Vec<Rc<RefCell<Chord>>>
}

impl Line {
    pub fn new() -> Self {
        Self { chords: Vec::new() }
    }
    /// defined as the minimum length of each chord
    pub fn offset(&self) -> usize {
        self.chords.iter().map(|ch| RefCell::borrow(ch).length).min().unwrap()
    }
    /// defined as the maximum size of each chord
    pub fn size(&self) -> usize {
        self.chords.iter().map(|ch| RefCell::borrow(ch).size).max().unwrap()
    }
    pub fn push(&mut self, chord: Rc<RefCell<Chord>>) {
        self.chords.push(chord);
    }
    pub fn chords(&self) -> Iter<Rc<RefCell<Chord>>> {
        self.chords.iter()
    }
}
//#endregion Line
