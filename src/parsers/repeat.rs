use std::cell::RefCell;
use std::collections::BTreeMap;
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
    let bytes = token.as_bytes();
    bytes.contains(&REPEAT) && bytes.iter().all(
        |&b| b.is_ascii_digit() || matches!(b, REPEAT | DELIM | SEP)
    )
}

/// return volta not found error to be handled
fn not_found(v: usize, action: &str) -> Result<(), String> {
    let volta = match v {
        0 => "pre-volta".to_string(),
        v if v == !0 => "post-volta".to_string(),
        _ => format!("volta no. {}", v),
    };
    Err(format!("{} is not found while trying to {}", volta, action))
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
    voltas: BTreeMap<usize, Rc<RefCell<Vec<Line>>>>,
    /// indices of one of the voltas to record
    current: usize,
}

impl RepeatParser {
    pub fn new() -> Self {
        Self {
            voltas: BTreeMap::new(),
            current: 0,
        }
    }
    /// parse token as repeat
    pub fn parse(&self, token: &str) -> Result<Rep, String> {
        let bytes = token.as_bytes();
        match bytes {
            &[REPEAT] => Ok(Rep::VoltaEnd),
            &[DELIM, REPEAT] => Ok(Rep::RepeatEnd),
            &[REPEAT, DELIM] => Ok(Rep::RepeatStart),
            // parse as volta start or die
            _ => if let Some(voltas) = parse_volta_start(bytes) {
                Ok(Rep::VoltaStart(voltas))
            } else {
                Err(format!("invalid token as repeat: {}", token))
            }
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
    pub fn push(&mut self, line: Line) -> Result<(), String> {
        if line.size() == 0 {
            return Err(format!("attempt to push empty line"));
        }
        match self.voltas.get(&self.current) {
            Some(volta) => Ok(volta.borrow_mut().push(line)),
            None => not_found(self.current, "push new line"),
        }
    }
    /// repeat voltas and reset self
    pub fn repeat(&self, mut write: impl FnMut(&Line) -> Result<(), String>) -> Result<(), String> {
        if self.voltas.len() > 2 {
            for &k in self.voltas.keys().filter(|&&k| 0 < k && k < !0) {
                // write pre-volta volta post-volta
                self.write(0, &mut write)?;
                self.write(k, &mut write)?;
                self.write(!0, &mut write)?;
            }
        } else {
            // no voltas, only pre-volta
            self.write(0, &mut write)?;
            self.write(0, &mut write)?;
        }
        Ok(())
    }
    /// free data
    pub fn clear(&mut self) {
        self.voltas.clear();
        self.current = 0;
    }
    /// write a volta
    fn write(&self, v: usize, write: &mut impl FnMut(&Line) -> Result<(), String>) -> Result<(), String> {
        match self.voltas.get(&v) {
            // didn't know that you can put a for loop inside Ok()
            Some(volta) => Ok(for line in volta.borrow().iter() { write(line)?; }),
            None => not_found(v, "write line"),
        }
    }
}

