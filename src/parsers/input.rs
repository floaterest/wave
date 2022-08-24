use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::SplitAsciiWhitespace;

use crate::parsers::capture::{Cap, CaptureParser, should_be_cap};
use crate::parsers::note::{Note, NoteParser};
use crate::parsers::repeat::{Rep, RepeatParser, should_be_rep};
use crate::stores::note::{Chord, Line};
use crate::stores::waveform::Waveform;
use crate::writer::Writer;

/// Length, Frequency, Capture, Front, None
#[derive(PartialEq)]
enum Token {
    Note(Note),
    Cap(Cap),
    None,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self {
            Self::Note(Note::Freq(_)) => "note frequency",
            Self::Note(Note::Len(_, _)) => "note length",
            Self::Cap(Cap::Cap(_)) => "cap capture",
            Self::Cap(Cap::Front(_)) => "cap front",
            Self::None => "EOL",
        })
    }
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
    pub fn write<I: Iterator<Item=String>>(&mut self, lines: I) -> Result<(), Error> {
        self.wr.start(self.wave.fps)?;
        // not using for loops here because CLion won't give me autocomplete
        // lines.for_each(|line| self.parse_line(line.trim()));
        lines.enumerate().for_each(
            |(i, line)| self.parse_line(line.trim()).unwrap_or_else(
                |why| panic!("on line {}, {}", i + 1, why)
            )
        );
        Ok(self.wr.finish()?)
    }
    /// parse a line from input
    fn parse_line(&mut self, line: &str) -> Result<(), String> {
        Ok(match line.parse() {
            // line containing single usize
            Ok(bpm) => self.wave.bpm = bpm,
            Err(..) => {
                let mut tokens = line.split_ascii_whitespace().peekable();
                match tokens.peek() {
                    Some(&token) if should_be_rep(token) => self.parse_repeat(tokens)?,
                    Some(&token) if should_be_chords(token) => self.parse_chords(tokens)?,
                    _ => { /* token is comment */ }
                }
            }
        })
    }
    /// write a line to file
    fn write_line(&mut self, line: &Line) -> Result<(), String> {
        self.wave.fold_with_line(line)?;
        Ok(self.wr.write(self.wave.drain(line.offset())).unwrap())
    }
}

/// parse repeat
impl InputParser {
    /// parse a line of input as repeat
    fn parse_repeat(&mut self, mut tokens: Peekable<SplitAsciiWhitespace>) -> Result<(), String> {
        // current token type
        let mut cty = Some(self.rep.parse(tokens.next().unwrap())?);
        while let Some(ty) = cty {
            // next token type
            let nty = tokens.next().and_then(|token| Some(self.rep.parse(token).ok()?));

            match &ty {
                Rep::RepeatStart => self.rep.start(&[0]),
                Rep::VoltaStart(vs) => self.rep.start(&vs),
                Rep::RepeatEnd | Rep::VoltaEnd => self.rep.start(&[!0]),
            }
            match (ty, &nty) {
                // change repeat trigger to VoltaEnd
                (Rep::RepeatEnd, Some(Rep::VoltaStart(_))) => self.rep.set_trigger(Rep::VoltaEnd)?,
                // if current is the repeat trigger
                (end, Some(Rep::RepeatStart) | None) if self.rep.get_trigger() == end => {
                    // move self.rep to rep
                    let rep = std::mem::take(&mut self.rep);
                    // now there's no borrowing 2 values from self at tho same time
                    rep.repeat(|line| Ok(self.write_line(line)?))?;
                    // move back
                    self.rep = rep;
                    // reset repeat
                    self.rep.clear();
                }
                _ => ()
            }
            cty = nty;
        }
        Ok(())
    }
}

/// parse chords
impl InputParser {
    /// parse a line of input as chords (and captures)
    fn parse_chords(&mut self, mut tokens: Peekable<SplitAsciiWhitespace>) -> Result<(), String> {
        let (mut chord, mut rc) = (Chord::new(), Rc::new(Chord::new()));
        let mut line = Line::new();
        // current token type
        let mut cty = self.chord_type(tokens.next().unwrap())?;

        while cty != Token::None {
            // next token type
            let nty = match tokens.next() {
                Some(token) => self.chord_type(token)?,
                None => Token::None,
            };
            match &cty {
                // update set of keys to capture
                Token::Cap(Cap::Cap(key)) => self.cap.will_capture(Rc::clone(&key)),
                // update current chord's length & size
                Token::Note(Note::Len(length, staccato)) => {
                    let length = self.wave.frame_count(*length);
                    chord.length = length;
                    chord.size = if *staccato { length * 2 } else { length };
                }
                // extend current chord from captures and update to_shift/to_clear
                // Token::Cap(Cap::Front(captured)) => if chord.is_new() && rc.is_new() {
                Token::Cap(Cap::Front(captured)) => if chord.can_be_replaced_by(captured) && rc.is_empty() {
                    rc = Rc::clone(captured)
                } else {
                    chord.extend(captured)
                },
                // push new frequency to current chord
                Token::Note(Note::Freq(frequency)) => chord.push(*frequency),
                _ => {}
            }
            // help how do I refactor this monstrosity
            match (&cty, &nty) {
                // ignore (Len, Front | Freq)
                (Token::Note(Note::Len(_, _)), Token::Cap(Cap::Front(_)) | Token::Note(Note::Freq(_))) => (),
                // error (Len, Len | Cap | None) | (Cap, Freq | None)
                (Token::Note(Note::Len(_, _)), _) | (Token::Cap(Cap::Cap(_)), Token::Note(Note::Freq(_)) | Token::None) => {
                    return Err(format!("invalid token sequence: ({}, {})", cty, nty));
                }
                // ignore (Freq, Freq) | (Front, Freq | Front)
                (Token::Note(Note::Freq(_)), Token::Note(Note::Freq(_))) => (),
                (Token::Cap(Cap::Front(_)), Token::Note(Note::Freq(_)) | Token::Cap(Cap::Front(_))) => (),
                // push to line and capture (Freq, Len | Cap | Front | None) | (Front, Len | Cap | None)
                (Token::Note(Note::Freq(_)) | Token::Cap(Cap::Front(_)), _) => {
                    let new = if chord.is_empty() {
                        rc
                    } else {
                        Rc::new(chord + (*rc).clone())
                    };
                    self.cap.capture(Rc::clone(&new));
                    line.push(new);
                    chord = Chord::new();
                    rc = Rc::new(Chord::new());
                }
                _ => (),
            }
            cty = nty
        }
        if self.rep.on_rec() {
            self.rep.push(line)?;
        } else {
            self.write_line(&line)?;
        }
        Ok(self.cap.update())
    }
    /// get specific type of chord token
    fn chord_type(&mut self, token: &str) -> Result<Token, String> {
        if let Some(cap) = self.cap.try_parse(token)? {
            Ok(Token::Cap(cap))
        } else if let Some(note) = self.note.try_parse(token)? {
            Ok(Token::Note(note))
        } else {
            Err(format!("Cannot recognise token's type: {}", token))
        }
    }
}