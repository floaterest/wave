use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::stores::note::Line;

// use crate::stores::repeat::Repeat;

const REPEAT: u8 = b'|';
const DELIM: u8 = b':';
const SEP: u8 = b'.';

#[derive(PartialEq)]
pub enum Rep {
    /// |:
    RepeatStart,
    /// e.g. |1.3.
    VoltaStart(Vec<usize>),
    /// :|
    RepeatEnd,
    /// |
    VoltaEnd,
}

/// check if a line should be parsed as repeat based on the first token
pub fn should_be_rep(token: &str) -> bool {
    matches!(token.as_bytes()[0], REPEAT | DELIM)
}

/// panic because volta not found
fn panic(v: usize, wanted: &str, action: &str) -> ! {
    if v == !0 {
        panic!("Volta No. MAX is not {} while trying to {}", wanted, action)
    } else {
        panic!("Volta No. {} is not {} while trying to {}", v, wanted, action)
    }
}

fn parse_volta_start(bytes: &[u8]) -> Option<Vec<usize>> {
    match bytes.strip_prefix(&[REPEAT]) {
        // Some(voltas) => Some(voltas.iter().filter(|&b| b != &SEP).collect()),
        Some(voltas) => Some(voltas.iter().filter_map(
            |&b| if b == SEP { None } else { Some(b as usize) }
        ).collect()),
        None => None
    }
}

#[derive(Default)]
pub struct RepeatParser {
    /// 0 for pre-volta, MAX for post-volta
    voltas: HashMap<usize, Rc<RefCell<Vec<Line>>>>,
    /// indices of one of the voltas to record
    current: usize,
}

impl RepeatParser {
    pub fn new() -> Self {
        Self {
            voltas: HashMap::new(),
            current: 0,
        }
    }
    pub fn parse(&self, token: &str) -> Rep {
        let bytes = token.as_bytes();
        match bytes {
            &[REPEAT] => Rep::VoltaEnd,
            &[DELIM, REPEAT] => Rep::RepeatEnd,
            &[REPEAT, DELIM] => Rep::RepeatStart,
            // parse as volta start or die
            _ => Rep::VoltaStart(parse_volta_start(bytes).unwrap_or_else(
                || panic!("Invalid token as repeat: {}", token)
            ))
        }
    }
    /// return if Repeat is currently recording
    pub fn on_rec(&self) -> bool {
        !self.voltas.is_empty()
    }
    /// init new voltas to store if empty
    pub fn start(&mut self, indices: &[usize]) {
        let volta = Rc::new(RefCell::new(Vec::new()));
        for &i in indices.iter() {
            self.voltas.entry(i).or_insert(Rc::clone(&volta));
        }
        self.current = indices[0];
    }
    /// add new line to current voltas
    pub fn push(&mut self, line: Line) {
        match self.voltas.get(&self.current) {
            Some(volta) => volta.borrow_mut().push(line),
            None => panic(self.current, "initialised", "push"),
        }
    }
    /// repeat voltas and reset self
    pub fn repeat(&self, mut write: impl FnMut(&Line)) {
        if self.voltas.len() > 2 {
            self.voltas.keys().filter(|&&k| 0 < k && k < !0).for_each(|&k| {
                // pre-volta volta post-volta
                self.write(0, &mut write);
                self.write(k, &mut write);
                self.write(!0, &mut write);
            });
        } else {
            // no voltas, only pre-volta
            self.write(0, &mut write);
            self.write(0, &mut write);
        }
    }
    /// free data
    pub fn clear(&mut self) {
        self.voltas.clear();
        self.current = 0;
    }
    /// write a volta
    fn write(&self, v: usize, write: &mut impl FnMut(&Line)) {
        match self.voltas.get(&v) {
            Some(volta) => volta.borrow().iter().for_each(|line| write(line)),
            None => panic(v, "found", "write"),
        }
    }
}

