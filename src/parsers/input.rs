use std::fs::File;
use std::io::Result;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::SplitAsciiWhitespace;

use crate::buffers::capture::Cap;
use crate::buffers::note::{Chord, Line, Note};
use crate::buffers::waveform::Waveform;
use crate::parsers::capture::{CAPTURE, CaptureParser};
use crate::parsers::note::{NoteParser, STACCATO};
use crate::writer::Writer;

/// check if a line should be parsed as chords based on the first token
fn should_be_chords(token: &str) -> bool {
    match token.as_bytes()[0] {
        CAPTURE => true,
        b if b.is_ascii_digit() => true,
        _ => false,
    }
}

enum Token {
    Note(Note),
    Capture(Cap),
    None,
}

pub struct InputParser {
    cap: CaptureParser,
    wr: Writer,
    note: NoteParser,
    wave: Waveform,
}

impl InputParser {
    pub fn new(output: File, amp: f64, fps: u32) -> Self {
        Self {
            wr: Writer::new(output),
            cap: CaptureParser::new(),
            note: NoteParser::new(),
            wave: Waveform::new(amp, fps),
        }
    }
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.wr.start(self.wave.fps)?;
        // not using for loops here because CLion won't give me autocomplete
        lines.for_each(|line| {
            let trim = line.trim();
            match trim.parse() {
                Ok(bpm) => self.wave.bpm = bpm,
                Err(..) => {
                    let mut tokens = trim.split_ascii_whitespace().peekable();
                    match tokens.peek() {
                        // Some(&token) if is_repeat(token) => println!("{} is repeat", token),
                        Some(&token) if should_be_chords(token) => self.parse_chords(tokens),
                        _ => { /* token is comment */ },
                    }
                }
            }
        });
        self.wr.write(self.wave.drain_all())?;
        Ok(self.wr.finish()?)
    }
    /// parse a line as chords (and captures)
    fn parse_chords(&mut self, mut tokens: Peekable<SplitAsciiWhitespace>) {
        let mut cty = self.chord_type(tokens.peek().unwrap());
        let mut chord = Chord::new();
        let mut line = Line::new();
        while let Some(token) = tokens.next() {
            let nty = match tokens.peek() {
                Some(&peek) => self.chord_type(peek),
                None => Token::None,
            };
            self.match_current_type(&cty, token, &mut chord);
            self.match_types(&cty, &nty, &mut chord, &mut line);
            cty = nty
        }
        self.wave.fold_with_line(&line);
        self.wr.write(self.wave.drain_until(line.offset())).unwrap();
        self.cap.update();
    }
    fn match_current_type(&mut self, cty: &Token, token: &str, chord: &mut Chord) {
        match cty {
            Token::Capture(Cap::Capture(key)) => {
                println!("Capture {}", key);
                self.cap.will_capture(Rc::new(key.clone()))
            },
            Token::Note(Note::Length(length)) => {
                let length = self.wave.frame_count(*length);
                chord.length = length;
                chord.size = if token.as_bytes().ends_with(&[STACCATO]) {
                    length * 2
                } else { length };
            }
            Token::Capture(Cap::Shift(key, clear)) => {
                println!("Extend {}", key);
                chord.extend(&self.cap.will_shift(Rc::new(key.clone()), *clear));
            }
            Token::Note(Note::Frequency(frequency)) => {
                chord.push(*frequency)
            }
            _ => {},
        }
    }
    fn match_types(&mut self, cty: &Token, nty: &Token, chord: &mut Chord, line: &mut Line) {
        match (cty, nty) {
            (Token::Note(Note::Frequency(_)), Token::Note(Note::Frequency(_))) => {},
            (Token::Capture(Cap::Shift(_, _)), Token::Capture(Cap::Shift(_, _)) | Token::Note(Note::Frequency(_))) => {},
            // (P, C|L|S) | (S,C|L) | (_, N)
            (Token::Note(Note::Frequency(_)) | Token::Capture(Cap::Shift(_, _)), _) | (_, Token::None) => {
                let rc = Rc::new((*chord).clone());
                self.cap.capture(&rc);
                (*line).push(rc);
                (*chord).clear();
            }
            _ => {},
        }
    }
    fn chord_type(&mut self, token: &str) -> Token {
        if let Some(cap) = self.cap.parse(token) {
            Token::Capture(cap)
        } else if let Some(note) = self.note.parse(token) {
            Token::Note(note)
        } else {
            panic!("Cannot recognise token's type: {}", token)
        }
    }
}