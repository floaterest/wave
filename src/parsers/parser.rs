use std::fs::File;
use std::io::Result;

use crate::line::{Chord, Line};
use crate::parsers::NoteParser;
use crate::repeat::Repeat;
use crate::waveform::Waveform;
use crate::writer::Writer;

pub const STACCATO: char = '*';
pub const REPEAT: char = '|';

pub struct Parser {
    writer: Writer,
    repeat: Repeat,
    wave: Waveform,
    note: NoteParser,
}

impl Parser {
    pub fn new(dest: File, fps: u32, amp: f64) -> Self {
        Self {
            wave: Waveform::new(amp, fps),
            writer: Writer::new(dest),
            repeat: Repeat::new(),
            note: NoteParser::new(),
        }
    }
    /// parse lines of input and write wave file
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.writer.start(self.wave.fps)?;
        let lines = lines.map(|line| line.trim().to_string()).filter(|line| line.len() > 0);
        for line in lines {
            self.parse(line)?;
        }
        self.writer.write(self.wave.drain_all())?;
        Ok(self.writer.finish()?)
    }
    /// parse line as input
    fn parse(&mut self, line: String) -> Result<()> {
        Ok(match line.split_whitespace() {
            sw if line.contains(REPEAT) => self.parse_repeat(sw),
            sw if line.chars().next().unwrap().is_ascii_digit() => match line.parse::<u16>() {
                Ok(bpm) => self.wave.bpm = bpm,
                Err(..) => self.parse_line(sw)?,
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
            Some(":") => self.repeat.start(&[0]),
            Some(s) => self.repeat.start(&s.split('.').filter(
                |ch| !ch.is_empty()
            ).flat_map(
                |ch| ch.parse()
            ).collect::<Vec<usize>>()),
            _ => panic!("Invalid repeat start token: {}", token),
        }
    }
    fn bton(&self, beat: f64) -> usize {
        //          duration in seconds        )     number of frames    )
        ((beat * 240.0 / self.wave.bpm as f64) * self.wave.fps as f64) as usize
    }
    fn parse_line<'a, I: Iterator<Item=&'a str>>(&mut self, tokens: I) -> Result<()> {
        let mut line = Line::new();
        let mut chord = Chord::new();
        tokens.for_each(|token| match token.chars().next() {
            // note length
            Some(ch) if ch.is_ascii_digit() => {
                if !chord.is_empty() {
                    line.push(chord.clone());
                    chord.clear();
                }
                let length = self.bton(Chord::parse_length(token));
                chord.length = length;
                chord.size = if token.ends_with(STACCATO) { length / 2 } else { length };
            },
            // note pitch
            Some(ch) if ch.is_ascii_alphabetic() => {
                chord.push(self.note.frequency(token));
            }
            _ => panic!("Invalid token as line of chords: {}", token),
        });
        line.push(chord);
        self.repeat.push(&line);
        self.wave.add_line(&line);
        Ok(self.writer.write(self.wave.drain_to(line.offset()))?)
    }
}