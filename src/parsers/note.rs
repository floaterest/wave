use std::collections::HashMap;

use crate::buffers::note::Note;

pub const TIE: u8 = b'+';
pub const DOTTED: u8 = b'.';
pub const STACCATO: u8 = b'*';
pub const REST: u8 = b'\\';
const TONES: [(&str, i32); 17] = [
    ("c", -8),
    ("c#", -7), ("db", -7),
    ("d", -6),
    ("d#", -5), ("eb", -5),
    ("e", -4),
    ("f", -3),
    ("f#", -2), ("gb", -2),
    ("g", -1),
    ("g#", 0), ("ab", 0),
    ("a", 1),
    ("a#", 2), ("bb", 2),
    ("b", 3),
];

pub struct NoteParser {
    notes: HashMap<usize, f64>,
    tones: HashMap<String, i32>,
}

impl NoteParser {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            tones: TONES.iter().map(|(t, i)| (t.to_string(), *i)).collect()
        }
    }
    pub fn parse(&mut self, token: &str) -> Option<Note> {
        if token.as_bytes()[0].is_ascii_digit() {
            Some(Note::Length(self.length(token)))
        } else if self.is_freq(token) {
            Some(Note::Frequency(self.frequency(token)))
        } else if self.is_rest(token) {
            Some(Note::Frequency(0.0))
        } else {
            None
        }
    }
    /// parse token as number of beats
    fn length(&self, token: &str) -> f64 {
        match token.parse::<usize>() {
            // normal note value
            Ok(length) => 1.0 / length as f64,
            Err(..) => match token.as_bytes().last() {
                Some(&DOTTED) => self.scale(token, 1.5, DOTTED),
                Some(&STACCATO) => self.scale(token, 0.5, STACCATO),
                // sum up each value
                _ if self.has_tie(token) => self.parse_tie(token),
                _ => panic!("Invalid token as node length: {}", token),
            }
        }
    }
    /// parse token as frequency
    fn frequency(&mut self, token: &str) -> f64 {
        // https://en.wikipedia.org/wiki/Piano_key_frequencies
        let key_num = self.key_number(token);
        *self.notes.entry(key_num).or_insert(2f64.powf((key_num as f64 - 49.0) / 12.0) * 440.0)
    }

    /// check if token is a frequency
    fn is_freq(&self, token: &str) -> bool {
        if token.is_empty() {
            false
        } else {
            self.tones.contains_key(&token[..token.len() - 1])
        }
    }
    /// check if token is rest
    fn is_rest(&self, token: &str) -> bool {
        token.as_bytes() == &[REST]
    }
    /// check of token as length has tie
    fn has_tie(&self, token: &str) -> bool {
        token.bytes().all(|ch| ch.is_ascii_digit() || ch == TIE)
    }
    /// parse token as tie
    fn parse_tie(&self, token: &str) -> f64 {
        token.split(TIE as char).flat_map(|s| s.parse::<f64>()).map(|f| 1.0 / f).sum()
    }
    /// scale the duration of a token as length
    fn scale(&self, token: &str, scale: f64, suffix: u8) -> f64 {
        scale / token.strip_suffix(suffix as char).unwrap().parse::<f64>().unwrap()
    }
    /// convert note to key number
    fn key_number(&self, note: &str) -> usize {
        let (tone, octave) = note.split_at(note.len() - 1);
        match (self.tones.get(tone), octave.parse::<i32>()) {
            (Some(i), Ok(oct)) => (i + oct * 12) as usize,
            _ => panic!("Invalid token as note: {}", note),
        }
    }
}
