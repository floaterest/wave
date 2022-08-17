use std::collections::VecDeque;
use std::rc::Rc;

use crate::stores::note::Chord;

pub struct Capture {
    /// cycle of chords
    captures: VecDeque<Rc<Chord>>,
}

impl Capture {
    pub fn new() -> Self {
        Self { captures: VecDeque::new() }
    }
    /// get current chord
    pub fn current(&self) -> Option<Rc<Chord>> {
        match self.captures.front() {
            Some(chord) => Some(Rc::clone(chord)),
            None => None,
        }
    }
    /// shift current to the next chord from capture
    pub fn shift(&mut self) -> Option<()> {
        match self.captures.pop_front() {
            Some(chord) => Some(self.captures.push_back(chord)),
            None => None,
        }
    }
    /// add a new chord to the end of Vec
    pub fn push(&mut self, chord: Rc<Chord>) {
        self.captures.push_back(chord);
    }
}
