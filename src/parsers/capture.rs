use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::stores::capture::{Cap, Capture};
use crate::stores::note::Chord;

const CAPTURE: u8 = b'(';
const SHIFT: u8 = b'[';
const CLEAR: u8 = b'{';

/// check if byte should be start of capture token
pub fn should_be_cap(byte: u8) -> bool {
    matches!(byte, CAPTURE | SHIFT | CLEAR)
}

/// panic key not found error
fn panic(key: &str, action: &str) -> ! {
    panic!("Key '{}' not found while trying to {}", key, action)
}

/// parse token as capture or die
fn parse_cap(token: &str, prefix: u8) -> Cap {
    // get matching pair
    let suffix = match prefix {
        CAPTURE => b')',
        SHIFT | CLEAR => prefix + 2,
        _ => panic!("Invalid token as capture: {}", token),
    };

    let good = token.bytes().all(
        |ch| ch == prefix || ch == suffix || ch.is_ascii_alphabetic()
    );
    assert!(good, "Invalid token as capture, {}", token);
    let key = token.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    match prefix {
        CAPTURE => Cap::Capture(key),
        SHIFT | CLEAR => Cap::Shift(key, prefix == CLEAR),
        _ => panic!("wot"),
    }
}

pub struct CaptureParser {
    /// stores the captured chords
    captures: HashMap<Rc<String>, Capture>,
    /// set of keys (rc pointing to keys in captures) to capture
    to_capture: HashSet<Rc<String>>,
    /// set of keys (rc pointing to keys in captures) to shift
    to_shift: HashSet<Rc<String>>,
    /// set of keys (rc pointing to keys in captures) to clear
    to_clear: HashSet<Rc<String>>,
}

impl CaptureParser {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
            to_capture: HashSet::new(),
            to_shift: HashSet::new(),
            to_clear: HashSet::new(),
        }
    }
    /// schedule key to be captured
    pub fn will_capture(&mut self, key: Rc<String>) {
        self.to_capture.insert(key);
    }
    /// schedule key to shift (or clear) and return current chord in key
    pub fn will_shift(&mut self, key: Rc<String>, clear: bool) -> Rc<Chord> {
        let set = if clear { &mut self.to_clear } else { &mut self.to_shift };
        match self.captures.get_key_value(&key) {
            Some((key, chord)) => {
                // insert the key from captures, not from parameters
                // (they are the same value but rc points to different places)
                (*set).insert(Rc::clone(key));
                chord.current()
            },
            None => panic(&key, "get current"),
        }
    }
    /// push a chord to captures and clear to_capture
    pub fn capture(&mut self, chord: &Rc<Chord>) {
        for cap in self.to_capture.drain() {
            (*self.captures.entry(cap).or_insert(Capture::new())).push(Rc::clone(chord))
        }
    }
    /// update cycle and clear all HashSets
    pub fn update(&mut self) {
        // assume to_capture is already cleared from self.capture()
        for shift in self.to_shift.iter() {
            match self.captures.get_mut(shift) {
                Some(chord) => chord.shift(),
                None => panic(shift, "shift"),
            }
        }
        for clear in self.to_clear.iter() {
            self.captures.remove(clear);
        }
        self.to_shift.clear();
        self.to_clear.clear();
    }
    /// parse token as capture
    pub fn parse(&self, token: &str) -> Option<Cap> {
        match token.as_bytes()[0] {
            b if should_be_cap(b) => Some(parse_cap(token, b)),
            _ => None,
        }
    }
}