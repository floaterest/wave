use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::line::Chord;

pub struct Capture {
    index: usize,
    captures: Vec<Rc<RefCell<Chord>>>,
}

impl Capture {
    pub fn new() -> Self {
        Self { index: 0, captures: Vec::new() }
    }
    pub fn current(&self) -> Rc<RefCell<Chord>> {
        Rc::clone(&self.captures[self.index])
    }
    pub fn shift(&mut self) {
        self.index = (self.index + 1) % self.captures.len();
    }
    pub fn push(&mut self, chord: Rc<RefCell<Chord>>) {
        self.captures.push(chord);
    }
}

pub struct CaptureMap {
    captures: HashMap<Rc<String>, Capture>,
    pub to_clear: HashSet<Rc<String>>,
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
    pub fn parse_key(token: &str) -> String {
        token.chars().filter(|ch| ch.is_ascii_alphabetic()).collect()
    }
    pub fn push(&mut self, key: Rc<String>, chord: Rc<RefCell<Chord>>) {
        let cap = self.captures.entry(key).or_insert(Capture::new());
        (*cap).push(chord);
    }
    pub fn current(&self, key: &String) -> Rc<RefCell<Chord>> {
        match self.captures.get(key) {
            Some(chord) => chord.current(),
            None => panic!("Key {} not found when trying to get current", key)
        }
    }
    pub fn update(&mut self) {
        for shift in self.to_shift.iter() {
            self.captures.get_mut(shift).unwrap().shift();
        }
        for clear in self.to_clear.iter() {
            self.captures.remove(clear);
        }
        self.to_shift.clear();
        self.to_clear.clear();
    }
}
