use std::collections::HashMap;
use std::fs::File;
use std::io::Result;

use crate::{DOTTED, Repeat, REPEAT, STACCATO, TIE};
use crate::note::ntof;
use crate::writer::Writer;

pub struct Wave {
    writer: Writer,
    repeat: Repeat,
    /// note -> freq
    notes: HashMap<String, f64>,
}

impl Wave {
    pub fn new(dest: File, rate: u32, amp: f64) -> Self {
        Self {
            writer: Writer::new(dest, rate, amp),
            repeat: Repeat::new(),
            notes: HashMap::new(),
        }
    }
    //#region parse input
    /// parse iter of lines
    pub fn parse<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.writer.start()?;
        let lines = lines.map(|line| line.trim().to_string()).filter(|line| line.len() > 0);
        for line in lines {
            self.parse_line(line)?;
        }
        Ok(self.writer.finish()?)
    }
    /// parse a line of input
    fn parse_line(&mut self, line: String) -> Result<()> {
        Ok(match line.split_whitespace() {
            sw if line.contains(REPEAT) => self.parse_repeat(sw),
            sw if line.chars().next().unwrap().is_ascii_digit() => match line.parse::<u16>() {
                Ok(bpm) => self.writer.bpm = bpm,
                Err(..) => self.parse_notes(sw)?,
            }
            _ => {},
        })
    }
    fn parse_repeat<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) {
        tokens.for_each(|token: &str| match token {
            _ if token.ends_with(REPEAT) => self.parse_end(token),
            _ if token.starts_with(REPEAT) => self.parse_start(token),
            _ => panic!("Invalid repeat token: {}", token)
        })
    }
    fn parse_end(&mut self, token: &str) {
        match token.strip_suffix(REPEAT) {
            // end all voltas
            Some("") => self.repeat.clear(),
            Some(":") => {
                self.rep();
                // if doesn't have voltas starting from 1
                if self.repeat.voltas.len() == 1 { self.repeat.clear(); }
            }
            _ => panic!("Invalid repeat end token: {}", token)
        }
    }
    pub fn rep(&mut self) {
        let writer = &mut self.writer;
        // append repeat
        self.repeat.voltas[0].iter().for_each(|line| writer.append_line(line));
        // ready to store next volta
        self.repeat.current += 1;
        // if the next volta is already stored (∴ won't appear in input)
        if self.repeat.voltas.len() > self.repeat.current && self.repeat.voltas[self.repeat.current].len() > 0 {
            // append current volta
            self.repeat.voltas[self.repeat.current].iter().for_each(|line| writer.append_line(line));
            // append repeat agait
            self.repeat.voltas[0].iter().for_each(|line| writer.append_line(line));
            self.repeat.current += 1;
        }
    }
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
        let mut size = self.writer.buffer.len();
        tokens.for_each(|token: &str| match token.chars().next() {
            // note length
            Some(b) if b.is_ascii_digit() => {
                len = self.parse_len(token);
                // need more space for this len
                if len > size {
                    size = len;
                    self.writer.buffer.resize(len, 0);
                }
                // always set offset as shortest len
                if offset == 0 || len < offset { offset = len; }
                // stacatto ==> half note length
                if token.ends_with(STACCATO) { len /= 2; }
            },
            // note value
            _ => {
                let freq = *self.notes.entry(token.to_string()).or_insert(ntof(token.as_bytes()));
                self.writer.append(len, freq);
                self.repeat.push(len, freq);
            },
        });
        self.repeat.flush(size, offset);
        Ok(self.writer.flush(offset)?)
    }
    /// parse a token as note length
    /// returns number of frames
    fn parse_len(&mut self, token: &str) -> usize {
        fn strip(token: &str, suffix: char, scale: f64) -> f64 {
            scale / token.strip_suffix(suffix).unwrap().parse::<f64>().unwrap()
        }
        let length = match token.parse::<f64>() {
            Ok(len) => 1.0 / len,
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
        ((length * 240.0 / self.writer.bpm as f64) * self.writer.rate as f64) as usize
    }
    //#endregion parse input
}