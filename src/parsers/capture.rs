use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::stores::capture::Capture;
use crate::stores::note::Chord;

const CAPTURE: u8 = b'(';
const SHIFT: u8 = b'[';
const CLEAR: u8 = b'{';
const RAISE: u8 = b'^';
const LOWER: u8 = b'_';

#[derive(PartialEq)]
pub enum Cap {
    /// (key)
    Capture(String),
    /// (key, clear, scale)
    Shift(String, bool, Option<f64>),
}

/// check if a line should be parsed as capture based on the first byte
pub fn should_be_cap(byte: u8) -> bool {
    matches!(byte, CAPTURE | SHIFT | CLEAR)
}

/// check if capture is cap
fn is_cap(bytes: &[u8], prefix: u8, suffix: u8) -> bool {
    bytes.iter().all(
        |&b| matches!(b, LOWER | RAISE) ||
            b == prefix || b == suffix ||
            b.is_ascii_alphanumeric()
    )
}

/// panic because key not found
fn panic_not_found(key: &str, action: &str) -> ! {
    panic!("Key '{}' not found while trying to {}", key, action)
}

/// panic because key found but value is empty
fn panic_empty(key: &str, action: &str) -> ! {
    panic!("Capture in '{}' is empty while trying to {}", key, action)
}

/// parse token as capture or die
fn parse_cap(token: &str, prefix: u8) -> Cap {
    // get matching pair
    let suffix = match prefix {
        CAPTURE => b')',
        SHIFT | CLEAR => prefix + 2,
        _ => panic!("Invalid token as capture: {}", token),
    };
    let bytes = token.as_bytes();
    assert!(is_cap(bytes, prefix, suffix), "Invalid token as capture, {}", token);
    let key = token.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    match prefix {
        CAPTURE => Cap::Capture(key),
        SHIFT | CLEAR => Cap::Shift(key, prefix == CLEAR, parse_scale(bytes)),
        _ => panic!("wot"),
    }
}

fn parse_scale(bytes: &[u8]) -> Option<f64> {
    if bytes.ends_with(&[LOWER, b'8']) {
        Some(0.5)
    } else if bytes.ends_with(&[RAISE, b'8']) {
        Some(2.0)
    } else {
        None
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
                match chord.current() {
                    Some(chord) => chord,
                    None => panic_empty(&key, "get current"),
                }
            },
            None => panic_not_found(&key, "get current"),
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
                Some(chord) => chord.shift().unwrap_or_else(
                    || panic_empty(shift, "shift")
                ),
                None => panic_not_found(shift, "shift"),
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