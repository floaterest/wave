use std::ops::Add;
use std::rc::Rc;
use std::slice::Iter;

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
    pub fn scale(&self, scale: f64) -> Self {
        Self {
            length: self.length,
            size: self.size,
            frequencies: self.frequencies.iter().map(|&f| f * scale).collect(),
        }
    }
    /// returns `true` if `self` can be replaced by chord
    pub fn can_be_replaced_by(&self, chord: &Chord) -> bool {
        self.is_empty()
            && (self.size == chord.size && self.length == chord.length)
            || (self.size == 0 && self.length == 0)
    }
    /// returns `true` if `self` has no frequencies
    pub fn is_empty(&self) -> bool {
        self.frequencies.is_empty()
    }
    /// push a new frequency to chord
    pub fn push(&mut self, frequency: f64) {
        self.frequencies.push(frequency);
        // f64 does not implement Eq ffs
        // assert!(self.frequencies.insert(frequency), "attempt to insert existing frequency to a chord: {}", frequency);
    }
    /// extend the frequencies in rhs to lhs
    pub fn extend(&mut self, rhs: &Rc<Chord>) {
        self.frequencies.extend(rhs.frequencies.iter());
    }
}

impl Add for Chord {
    type Output = Self;

    /// create new chord with length and size from lhs and frequencies from both
    /// (thus the operation is not commutative)
    fn add(self, rhs: Self) -> Self::Output {
        if rhs.is_empty() {
            Self { ..self }
        } else {
            let mut frequencies = self.frequencies.clone();
            frequencies.extend(&rhs.frequencies);
            Self { frequencies, ..self }
        }
    }
}

impl PartialEq for Chord {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.length == other.length && self.frequencies == other.frequencies
    }
}
//#endregion  Chord

//#region Line
/// collections of chords to be played at the same time
#[derive(Clone, Debug)]
pub struct Line {
    chords: Vec<Rc<Chord>>,
}

impl Line {
    pub fn new() -> Self {
        Self { chords: Vec::new() }
    }
    /// defined as the minimum size of each chord
    pub fn offset(&self) -> usize {
        self.chords.iter().map(|ch| ch.size).min().unwrap()
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
