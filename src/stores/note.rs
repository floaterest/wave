use std::rc::Rc;
use std::slice::Iter;

#[derive(PartialEq)]
pub enum Note {
    /// (number of beats, staccato)
    Length(f64, bool),
    /// Hz
    Frequency(f64),
}

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
    pub fn new() -> Self {
        Self {
            length: 0,
            size: 0,
            frequencies: Vec::new(),
        }
    }
    /// scale all frequencies, return new Self
    // pub fn scale(&self, scale: f64) -> Self {
    //     Self {
    //         length: self.length,
    //         size: self.size,
    //         frequencies: self.frequencies.iter().map(|&f| f * scale).collect()
    //     }
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
            assert_eq!(self.length, rhs.length, "attempt to extend a chord without equal length");
            assert_eq!(self.size, rhs.size, "attempt to extend a chord without equal size");
            self.frequencies.extend(rhs.frequencies.iter());
        }
    }
    /// reset itself
    pub fn clear(&mut self) {
        (self.length, self.size) = (0, 0);
        self.frequencies.clear();
    }
}
//#endregion  Chord

//#region Line
/// collections of chords to be played at the same time
#[derive(Clone, Debug)]
pub struct Line {
    chords: Vec<Rc<Chord>>
}

impl Line {
    pub fn new() -> Self {
        Self { chords: Vec::new() }
    }
    /// defined as the minimum length of each chord
    pub fn offset(&self) -> usize {
        self.chords.iter().map(|ch| ch.length).min().unwrap()
    }
    /// defined as the maximum size of each chord
    pub fn size(&self) -> usize {
        self.chords.iter().map(|ch| ch.size).max().unwrap()
    }
    pub fn push(&mut self, chord: Rc<Chord>) {
        self.chords.push(chord);
    }
    pub fn chords(&self) -> Iter<Rc<Chord>> {
        self.chords.iter()
    }
}
//#endregion Line
