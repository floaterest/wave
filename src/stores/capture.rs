use std::rc::Rc;

use crate::stores::note::Chord;

#[derive(PartialEq)]
pub enum Cap {
    /// (key)
    Capture(String),
    /// (key, clear)
    Shift(String, bool),
}

pub struct Capture {
    /// current chord
    index: usize,
    /// cycle of chords
    captures: Vec<Rc<Chord>>,
}

impl Capture {
    pub fn new() -> Self {
        Self { index: 0, captures: Vec::new() }
    }
    /// get current chord
    pub fn current(&self) -> Rc<Chord> {
        Rc::clone(&self.captures[self.index])
    }
    /// shift current to the next chord from capture
    pub fn shift(&mut self) {
        self.index = (self.index + 1) % self.captures.len();
    }
    /// add a new chord to the end of Vec
    pub fn push(&mut self, chord: Rc<Chord>) {
        self.captures.push(chord);
    }
}
