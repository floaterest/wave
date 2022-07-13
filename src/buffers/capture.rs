use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::buffers::Chord;

//#region Capture
pub struct Capture {
    /// current chord
    index: usize,
    /// cycle of chords
    captures: Vec<Rc<RefCell<Chord>>>,
}

impl Capture {
    pub fn new() -> Self {
        Self { index: 0, captures: Vec::new() }
    }
    /// get current chord
    pub fn get(&self) -> Rc<RefCell<Chord>> {
        Rc::clone(&self.captures[self.index])
    }
    /// shift current to the next chord from capture
    pub fn shift(&mut self) {
        self.index = (self.index + 1) % self.captures.len();
    }
    /// add a new chord to the end of Vec
    pub fn push(&mut self, chord: Rc<RefCell<Chord>>) {
        self.captures.push(chord);
    }
}
//#endregion Capture

//#region CaptureMap
pub struct CaptureMap {
    captures: HashMap<Rc<String>, Capture>,
    /// contains keys in `captures`
    pub to_clear: HashSet<Rc<String>>,
    /// contains keys in `captures`
    pub to_shift: HashSet<Rc<String>>,
}

impl CaptureMap {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
            to_clear: HashSet::new(),
            to_shift: HashSet::new(),
        }
    }
    /// get key from a token
    pub fn parse_key(token: &str) -> String {
        token.chars().filter(|ch| ch.is_ascii_alphabetic()).collect()
    }
    /// push a chord to a capture by key
    pub fn push_by_key(&mut self, key: Rc<String>, chord: Rc<RefCell<Chord>>) {
        (*self.captures.entry(key).or_insert(Capture::new())).push(chord);
    }

    /// get current chord from a capture by key
    pub fn get_by_key(&self, key: &String) -> Rc<RefCell<Chord>> {
        match self.captures.get(key) {
            Some(chord) => chord.get(),
            None => panic!("Key '{}' not found when trying to get", key),
        }
    }
    /// update cycle and reset to_shift and to_clear
    pub fn update(&mut self) {
        for shift in self.to_shift.iter() {
            match self.captures.get_mut(shift) {
                Some(sh) => sh.shift(),
                None => panic!("Key '{}' not found when trying to shift", shift),
            }
        }
        for clear in self.to_clear.iter() {
            self.captures.remove(clear);
        }
        self.to_shift.clear();
        self.to_clear.clear();
    }
}
//#endregion CaptureMap
