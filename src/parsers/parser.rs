use std::cell::RefCell;
use std::fs::File;
use std::io::Result;
use std::rc::Rc;

use crate::buffers::{CaptureMap, Chord, Line, Repeat, Waveform};
use crate::parsers::NoteParser;
use crate::writer::Writer;

const STACCATO: char = '*';
const REPEAT: char = '|';
const CAPTURE: &str = "([{";

/// is not a comment
fn valid(ch: char) -> bool {
    ch.is_ascii_digit() || CAPTURE.contains(ch)
}

pub struct InputParser {
    note: NoteParser,
    wave: Waveform,
    writer: Writer,
    repeat: Repeat,
    capture: CaptureMap,
}

impl InputParser {
    pub fn new(output: File, fps: u32, amp: f64) -> Self {
        Self {
            wave: Waveform::new(amp, fps),
            writer: Writer::new(output),
            repeat: Repeat::new(),
            note: NoteParser::new(),
            capture: CaptureMap::new(),
        }
    }
    /// parse lines of input and write wave file
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.writer.start(self.wave.fps)?;
        // only parse non empty lines
        lines.filter_map(|line| match line.trim() {
            trim if trim.len() > 0 => Some(trim.to_string()),
            _ => None,
        }).for_each(|line| self.parse_line(line));
        // empty buffer to wave
        self.writer.write(self.wave.drain_all())?;
        Ok(self.writer.finish()?)
    }
    /// parse a line of input
    fn parse_line(&mut self, line: String) {
        match line.split_whitespace() {
            // parse as repeat
            sw if line.contains(REPEAT) => self.parse_repeat(sw),
            // parse as chords or captures
            sw if valid(line.chars().next().unwrap()) => match line.parse() {
                Ok(bpm) => self.wave.bpm = bpm,
                Err(..) => self.parse_chords(sw),
            }
            _ => { /* ignore comment */ },
        };
    }
    /// parse line as repeat instructions
    fn parse_repeat<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) {
        tokens.for_each(|token: &str| match token {
            _ if token.ends_with(REPEAT) => self.parse_end(token),
            _ if token.starts_with(REPEAT) => self.parse_start(token),
            _ => panic!("Invalid repeat token: {}", token)
        })
    }
    /// parse token as repeat end
    fn parse_end(&mut self, token: &str) {
        match token.strip_suffix(REPEAT) {
            // end all voltas
            Some("") => self.repeat.clear(),
            Some(":") => {
                self.repeat.repeat(&mut self.wave, &mut self.writer);
                // if doesn't have voltas starting from 1
                if self.repeat.voltas.len() == 1 { self.repeat.clear(); }
            }
            _ => panic!("Invalid repeat end token: {}", token)
        }
    }
    /// parse token as repeat start
    fn parse_start(&mut self, token: &str) {
        match token.strip_prefix(REPEAT) {
            // start new repeat
            Some(":") => self.repeat.start(&[0]),
            // start voltas
            Some(s) => self.repeat.start(&s.split('.').filter(
                |ch| !ch.is_empty()
            ).flat_map(|ch| ch.parse()).collect::<Vec<usize>>()),
            _ => panic!("Invalid repeat start token: {}", token),
        }
    }
    /// parse line as chords or captures
    fn parse_chords<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) {
        let mut line = Line::new();
        let mut chord = Rc::new(RefCell::new(Chord {
            size: 0,
            length: 0,
            frequencies: Vec::new()
        }));
        tokens.for_each(|token: &str| match token.chars().next() {
            // note length
            Some(ch) if ch.is_ascii_digit() => {
                // make new chord and push to line
                let length = self.wave.frame_count(Chord::parse_length(token));
                let size = if token.ends_with(STACCATO) { length / 2 } else { length };
                chord = Rc::new(RefCell::new(Chord {
                    length,
                    size,
                    frequencies: Vec::new()
                }));
                line.push(Rc::clone(&chord));
            }
            // note pitch
            Some(ch) if ch.is_ascii_alphabetic() => {
                chord.borrow_mut().push(self.note.frequency(token));
            }
            // capture
            Some(ch) if CAPTURE.contains(ch) => {
                let key = Rc::new(CaptureMap::parse_key(token));
                match ch {
                    '(' => {
                        self.capture.push_by_key(Rc::clone(&key), Rc::clone(&chord))
                    },
                    '[' | '{' => {
                        let captured = self.capture.get_by_key(&key);
                        chord.borrow_mut().extend(&captured.borrow());
                        line.push(Rc::clone(&chord));
                        let to = if ch == '[' {
                            &mut self.capture.to_shift
                        } else {
                            &mut self.capture.to_clear
                        };
                        to.insert(key);
                    }
                    _ => panic!("Invalid token as capture: {}", token),
                }
            }
            _ => panic!("Invalid token as line of chords: {}", token),
        });
        // write to file
        self.wave.fold_with_line(&line);
        self.writer.write(self.wave.drain_until(line.offset())).unwrap();
        // update repeat and capture
        self.repeat.push(line);
        self.capture.update();
    }
}