use std::fs::File;
use std::io::Result;

use crate::buffer::Buffer;
use crate::parsers::NoteParser;
use crate::repeat::Repeat;
use crate::writer::Writer;

pub const DOTTED: char = '.';
pub const STACCATO: char = '*';
pub const TIE: char = '+';
pub const REPEAT: char = '|';

pub struct Parser {
    writer: Writer,
    repeat: Repeat,
    buffer: Buffer,
    note: NoteParser,
}

impl Parser {
    pub fn new(dest: File, fps: u32, amp: f64) -> Self {
        Self {
            buffer: Buffer::new(amp, fps),
            writer: Writer::new(dest),
            repeat: Repeat::new(),
            note: NoteParser::new(),
        }
    }
    /// parse lines of input and write wave file
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.writer.start(self.buffer.fps)?;
        let lines = lines.map(|line| line.trim().to_string()).filter(|line| line.len() > 0);
        for line in lines {
            self.parse_line(line)?;
        }
        self.writer.write(self.buffer.drain(self.buffer.len()))?;
        Ok(self.writer.finish()?)
    }
    /// parse line as input
    fn parse_line(&mut self, line: String) -> Result<()> {
        Ok(match line.split_whitespace() {
            sw if line.contains(REPEAT) => self.parse_repeat(sw),
            sw if line.chars().next().unwrap().is_ascii_digit() => match line.parse::<u16>() {
                Ok(bpm) => self.buffer.bpm = bpm,
                Err(..) => self.parse_notes(sw)?,
            }
            _ => {},
        })
    }
    /// parse line as repeat instructions
    fn parse_repeat<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) {
        tokens.for_each(|token: &str| match token {
            _ if token.ends_with(REPEAT) => self.parse_end(token),
            _ if token.starts_with(REPEAT) => self.parse_start(token),
            _ => panic!("Invalid repeat token: {}", token)
        })
    }
    /// parse tokenas repeat end
    fn parse_end(&mut self, token: &str) {
        match token.strip_suffix(REPEAT) {
            // end all voltas
            Some("") => self.repeat.clear(),
            Some(":") => {
                self.repeat.repeat(&mut self.buffer, &mut self.writer);
                // if doesn't have voltas starting from 1
                if self.repeat.voltas.len() == 1 { self.repeat.clear(); }
            }
            _ => panic!("Invalid repeat end token: {}", token)
        }
    }
    /// parse token as repeat start
    fn parse_start(&mut self, token: &str) {
        match token.strip_prefix(REPEAT) {
            Some(":") => self.repeat.start(&[0]),
            Some(s) => self.repeat.start(&s.split('.').filter(
                |ch| !ch.is_empty()
            ).flat_map(
                |ch| ch.parse()
            ).collect::<Vec<usize>>()),
            _ => panic!("Invalid repeat start token: {}", token),
        }
    }
    /// parse a line of input as a note
    fn parse_notes<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) -> Result<()> {
        let (mut offset, mut len) = (0, 0);
        let mut size = self.buffer.len();
        tokens.for_each(|token: &str| match token.chars().next() {
            // note length
            Some(b) if b.is_ascii_digit() => {
                len = self.parse_len(token);
                // need more space for this len
                if len > size {
                    size = len;
                    self.buffer.resize(size);
                }
                // always set offset as shortest len
                if offset == 0 || len < offset { offset = len; }
                // stacatto ==> half note length
                if token.ends_with(STACCATO) { len /= 2; }
            },
            // note value
            _ => {
                let freq = self.note.frequency(token);
                self.buffer.add(len, freq);
                self.repeat.push(len, freq);
            },
        });
        self.repeat.flush(size, offset);
        Ok(self.writer.write(self.buffer.drain(offset))?)
    }
    /// parse a token as note length
    /// returns number of frames
    fn parse_len(&mut self, token: &str) -> usize {
        fn strip(token: &str, suffix: char, scale: f64) -> f64 {
            scale / token.strip_suffix(suffix).unwrap().parse::<f64>().unwrap()
        }
        let length = match token.parse::<usize>() {
            Ok(value) => 1.0 / value as f64,
            Err(..) => match token.chars().last() {
                Some(DOTTED) => strip(token, DOTTED, 1.5),
                // the actual node length will be parsed later
                Some(STACCATO) => strip(token, STACCATO, 1.0),
                _ if token.chars().all(
                    |ch| ch.is_ascii_digit() || ch == TIE
                ) => token.split(TIE).flat_map(|s| s.parse::<f64>()).map(|f| 1.0 / f).sum(),
                _ => panic!("Unknown token as node length: {}", token)
            }
        };
        ((length * 240.0 / self.buffer.bpm as f64) * self.buffer.fps as f64) as usize
    }
}