use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use crate::stores::note::Chord;

const CAP: u8 = b'(';
const POP: u8 = b'<';
const FRONT: u8 = b'[';
const CLEAR: u8 = b'{';
const ROTATE: u8 = b'|';

const RAISE: [u8; 2] = [b'^', b'8'];
const LOWER: [u8; 2] = [b'_', b'8'];

#[derive(PartialEq)]
pub enum Cap {
    /// (key)
    Cap(Rc<String>),
    /// (chord from capture)
    Front(Rc<Chord>),
}

/// check if a line should be parsed as capture based on the first byte
pub fn should_be_cap(byte: u8) -> bool {
    matches!(byte, CAP | POP | FRONT | CLEAR | ROTATE)
}

fn parse_scale(bytes: &[u8]) -> Option<f64> {
    if bytes.ends_with(&LOWER) {
        Some(0.5)
    } else if bytes.ends_with(&RAISE) {
        Some(2.0)
    } else {
        None
    }
}

pub struct CaptureParser {
    /// stores the captured chords
    captures: HashMap<Rc<String>, VecDeque<Rc<Chord>>>,
    /// things to do upon update
    to_cap: HashSet<Rc<String>>,
    to_pop: HashSet<Rc<String>>,
    to_clear: HashSet<Rc<String>>,
    to_rotate: HashSet<Rc<String>>,
}

impl CaptureParser {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
            to_cap: HashSet::new(),
            to_pop: HashSet::new(),
            to_clear: HashSet::new(),
            to_rotate: HashSet::new(),
        }
    }
    /// push new key to capture upon update
    pub fn will_capture(&mut self, key: Rc<String>) {
        self.to_cap.insert(key);
    }
    /// push a chord to captures and clear the keys to capture
    pub fn capture(&mut self, chord: Rc<Chord>) {
        let captures = &mut self.captures;
        self.to_cap.drain().for_each(
            |cap| captures.entry(cap).or_insert(VecDeque::new()).push_back(Rc::clone(&chord))
        );
    }
    /// update the captures
    pub fn update(&mut self) {
        let captures = &mut self.captures;

        let pop = &self.to_pop;
        let clear = &self.to_clear;
        let rotate = &self.to_rotate;
        // pop \ (shift ∪ clear)
        pop.difference(&rotate).filter(
            |&k| !clear.contains(k)
        ).for_each(|k| { captures.get_mut(k).unwrap().pop_front(); });
        // shift \ clear
        rotate.difference(&clear).for_each(
            |k| captures.get_mut(k).unwrap().rotate_left(1)
        );
        // kill the captures that were sentenced to death
        clear.iter().for_each(|k| { captures.remove(k); });
        // clear everything like nothing happened
        self.to_pop.clear();
        self.to_clear.clear();
        self.to_rotate.clear();
    }
    /// try parse token as capture
    pub fn try_parse(&mut self, token: &str) -> Result<Option<Cap>, String> {
        match token.as_bytes()[0] {
            b if should_be_cap(b) => Ok(Some(self.process(token, b)?)),
            _ => Ok(None),
        }
    }
    /// process the token as capture and return what operation was done
    fn process(&mut self, token: &str, prefix: u8) -> Result<Cap, String> {
        let key = token.chars().filter(|ch| ch.is_alphabetic()).collect();
        // use the key from captures if possible (avoid dup memory)
        let key = match self.captures.get_key_value(&key) {
            Some((k, _)) => Rc::clone(k),
            None => Rc::new(key),
        };
        match prefix {
            CAP => Ok(Cap::Cap(key)),
            POP | FRONT | CLEAR | ROTATE => Ok(self.process_front(
                key, prefix,
                parse_scale(token.as_bytes()),
            )?),
            _ => Err(format!("unknown capture instruction: {}", token)),
        }
    }
    /// process the token as pop/front/shift/clear
    fn process_front(&mut self, key: Rc<String>, prefix: u8, scale: Option<f64>) -> Result<Cap, String> {
        // update schedule
        let to = match prefix {
            CAP => Some(&mut self.to_cap),
            POP => Some(&mut self.to_pop),
            CLEAR => Some(&mut self.to_clear),
            ROTATE => Some(&mut self.to_rotate),
            _ => None,
        };
        if let Some(to) = to {
            to.insert(Rc::clone(&key));
        }
        // get front chord
        let front = Rc::clone(self.captures.get(&key).ok_or_else(
            || format!("key {} not found while trying to access front", &key)
        )?.front().ok_or_else(
            || format!("captures with key {} is empty while trying to access front", &key)
        )?);
        // if scale, make new rc
        Ok(Cap::Front(if let Some(r) = scale { Rc::new(front.scale(r)) } else { front }))
    }
}
