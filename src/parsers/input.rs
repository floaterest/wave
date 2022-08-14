use std::fs::File;
use std::io::Result;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::SplitAsciiWhitespace;

use crate::parsers::capture::{CaptureParser, should_be_cap};
use crate::parsers::note::NoteParser;
use crate::stores::capture::Cap;
use crate::stores::note::{Chord, Line, Note};
use crate::stores::waveform::Waveform;
use crate::writer::Writer;

/// check if a line should be parsed as chords based on the first token
fn should_be_chords(token: &str) -> bool {
    // check if line starts with
    match token.as_bytes()[0] {
        // capture tokens
        b if should_be_cap(b) => true,
        // note length
        b if b.is_ascii_digit() => true,
        _ => false,
    }
}

#[derive(PartialEq)]
enum Token {
    Note(Note),
    Capture(Cap),
    None,
}

pub struct InputParser {
    wr: Writer,
    cap: CaptureParser,
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
                // line containing single usize
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
        // write the rest of generated waveform
        self.wr.write(self.wave.drain_all())?;
        Ok(self.wr.finish()?)
    }
    /// parse a line as chords (and captures)
    fn parse_chords(&mut self, mut tokens: Peekable<SplitAsciiWhitespace>) {
        let mut chord = Chord::new();
        let mut line = Line::new();
        // current token type
        let mut cty = self.chord_type(tokens.next().unwrap());

        while cty != Token::None {
            // next token type
            let nty = match tokens.next() {
                Some(token) => self.chord_type(token),
                None => Token::None,
            };
            self.match_current_type(&cty, &mut chord);
            self.match_types(&cty, &nty, &mut chord, &mut line);
            cty = nty
        }
        self.wave.fold_with_line(&line);
        self.wr.write(self.wave.drain_until(line.offset())).unwrap();
        self.cap.update();
    }
    /// do stuff based on current type
    fn match_current_type(&mut self, cty: &Token, chord: &mut Chord) {
        match cty {
            // update set of keys to capture
            Token::Capture(Cap::Capture(key)) => {
                self.cap.will_capture(Rc::new(key.clone()))
            },
            // update current chord's length & size
            Token::Note(Note::Length(length, staccato)) => {
                let length = self.wave.frame_count(*length);
                chord.length = length;
                chord.size = if *staccato { length * 2 } else { length };
            }
            // extend current chord from captures and update to_shift/to_clear
            Token::Capture(Cap::Shift(key, clear)) => {
                chord.extend(&self.cap.will_shift(Rc::new(key.clone()), *clear));
            },
            // push new frequency to current chord
            Token::Note(Note::Frequency(frequency)) => {
                chord.push(*frequency)
            }
            _ => {},
        }
    }
    /// do stuff based on what comes next
    fn match_types(&mut self, cty: &Token, nty: &Token, chord: &mut Chord, line: &mut Line) {
        match (cty, nty) {
            (Token::Note(Note::Frequency(_)), Token::Note(Note::Frequency(_))) => (),
            (Token::Capture(Cap::Shift(_, _)), Token::Capture(Cap::Shift(_, _)) | Token::Note(Note::Frequency(_))) => (),
            // (Frequency, Capture) | (Frequency, Length) | (Frequency, Shift)
            // (Shift, Capture) | (Shift | Length)
            // (_ , None)
            (Token::Note(Note::Frequency(_)) | Token::Capture(Cap::Shift(_, _)), _) | (_, Token::None) => {
                let rc = Rc::new((*chord).clone());
                self.cap.capture(&rc);
                (*line).push(rc);
                (*chord).clear();
            }
            _ => (),
        }
    }
    /// get token type given token is from chords
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