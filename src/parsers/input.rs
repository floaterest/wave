use std::fs::File;
use std::io::Result;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::SplitAsciiWhitespace;

use crate::parsers::capture::{Cap, CaptureParser, should_be_cap};
use crate::parsers::note::{Note, NoteParser};
use crate::parsers::repeat::{Rep, RepeatParser, should_be_rep};
use crate::stores::note::{Chord, Line};
use crate::stores::waveform::Waveform;
use crate::writer::Writer;

#[derive(PartialEq)]
enum Token {
    Note(Note),
    Capture(Cap),
    Repeat(Rep),
    None,
}

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

pub struct InputParser {
    wr: Writer,
    cap: CaptureParser,
    rep: RepeatParser,
    note: NoteParser,
    wave: Waveform,
}

impl InputParser {
    pub fn new(output: File, amp: f64, fps: u32) -> Self {
        Self {
            wr: Writer::new(output),
            cap: CaptureParser::new(),
            rep: RepeatParser::new(),
            note: NoteParser::new(),
            wave: Waveform::new(amp, fps),
        }
    }
    /// parse all lines as input and write output to file
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<()> {
        self.wr.start(self.wave.fps)?;
        // not using for loops here because CLion won't give me autocomplete
        lines.for_each(|line| self.parse_line(line.trim()));
        Ok(self.wr.finish()?)
    }
    /// parse a line from input
    fn parse_line(&mut self, line: &str) {
        match line.parse() {
            // line containing single usize
            Ok(bpm) => self.wave.bpm = bpm,
            Err(..) => {
                let mut tokens = line.split_ascii_whitespace().peekable();
                match tokens.peek() {
                    Some(&token) if should_be_rep(token) => self.parse_repeat(tokens),
                    Some(&token) if should_be_chords(token) => self.parse_chords(tokens),
                    _ => { /* token is comment */ },
                }
            }
        }
    }
    /// write a line to file
    fn write_line(&mut self, line: &Line) {
        self.wave.fold_with_line(line);
        self.wr.write(self.wave.drain(line.offset())).unwrap();
    }
}

/// parse repeat
impl InputParser {
    /// parse a line of input as repeat
    fn parse_repeat(&mut self, mut tokens: Peekable<SplitAsciiWhitespace>) {
        let mut cty = self.repeat_type(tokens.next().unwrap());
        while cty != Token::None {
            // next token type
            let nty = match tokens.next() {
                Some(token) => self.repeat_type(token),
                None => Token::None,
            };
            self.match_repeat_type(&cty);
            self.match_repeat_types(&cty, &nty);
            cty = nty;
        }
    }
    /// do stuff based on current repeat type
    fn match_repeat_type(&mut self, cty: &Token) {
        match cty {
            Token::Repeat(Rep::RepeatStart) => self.rep.start(&[0]),
            Token::Repeat(Rep::VoltaStart(vs)) => self.rep.start(&vs),
            Token::Repeat(Rep::RepeatEnd | Rep::VoltaEnd) => self.rep.start(&[!0]),
            _ => (),
        }
    }
    /// do stuff based on what repeat token comes next
    fn match_repeat_types(&mut self, cty: &Token, nty: &Token) {
        match (cty, nty) {
            (Token::Repeat(Rep::RepeatStart | Rep::VoltaStart(_)), _) => (),
            (_, Token::Repeat(Rep::RepeatStart) | Token::None) => {
                // move self.rep to rep
                let rep = std::mem::take(&mut self.rep);
                // now there's no borrowing 2 values from self at tho same time
                rep.repeat(|line| self.write_line(line));
                // move back
                self.rep = rep;
                // reset repeat
                self.rep.clear();
            },
            _ => {},
        }
    }
    /// get specific type of repeat token
    fn repeat_type(&self, token: &str) -> Token {
        Token::Repeat(self.rep.parse(token))
    }
}

/// parse chords
impl InputParser {
    /// parse a line of input as chords (and captures)
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
            self.match_chord_type(&cty, &mut chord);
            self.match_chord_types(&cty, &nty, &mut chord, &mut line);
            cty = nty
        }
        if self.rep.on_rec() {
            self.rep.push(line);
        } else {
            self.write_line(&line);
        }
        self.cap.update();
    }
    /// do stuff based on current chord token
    fn match_chord_type(&mut self, cty: &Token, chord: &mut Chord) {
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
            Token::Capture(Cap::Shift(key, clear, scale)) => {
                let captured = self.cap.will_shift(Rc::new(key.clone()), *clear);
                chord.extend(if let Some(r) = scale {
                    Rc::new(captured.scale(*r))
                } else {
                    Rc::clone(&captured)
                });
            },
            // push new frequency to current chord
            Token::Note(Note::Frequency(frequency)) => chord.push(*frequency),
            _ => {},
        }
    }
    /// do stuff based on what chord token comes next
    fn match_chord_types(&mut self, cty: &Token, nty: &Token, chord: &mut Chord, line: &mut Line) {
        match (cty, nty) {
            (Token::Note(Note::Frequency(_)), Token::Note(Note::Frequency(_))) => (),
            (Token::Capture(Cap::Shift(..)), Token::Capture(Cap::Shift(..)) | Token::Note(Note::Frequency(_))) => (),
            // (Frequency, Capture) | (Frequency, Length) | (Frequency, Shift)
            // (Shift, Capture) | (Shift | Length)
            // (_ , None)
            (Token::Note(Note::Frequency(_)) | Token::Capture(Cap::Shift(..)), _) | (_, Token::None) => {
                let rc = Rc::new((*chord).clone());
                self.cap.capture(&rc);
                (*line).push(rc);
                (*chord).clear();
            }
            _ => (),
        }
    }
    /// get specific type of chord token
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