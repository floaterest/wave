use std::collections::HashMap;

const TIE: u8 = b'+';
const DOTTED: u8 = b'.';
const STACCATO: u8 = b'*';
const REST: u8 = b'\\';
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

#[derive(PartialEq)]
pub enum Note {
    /// (number of beats, staccato)
    Len(f64, bool),
    /// Hz
    Pitch(f64),
}

pub struct NoteParser {
    notes: HashMap<usize, f64>,
    tones: HashMap<String, i32>,
}

impl NoteParser {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            tones: TONES.iter().map(|(t, i)| (t.to_string(), *i)).collect(),
        }
    }
    /// try parse token as note length or frequency
    pub fn try_parse(&mut self, token: &str) -> Result<Option<Note>, String> {
        Ok(if token.as_bytes()[0].is_ascii_digit() {
            let (beat, staccato) = self.length(token)?;
            Some(Note::Len(beat, staccato))
        } else if self.is_pitch(token) {
            Some(Note::Pitch(self.frequency(token)?))
        } else if self.is_rest(token) {
            Some(Note::Pitch(0.0))
        } else {
            None
        })
    }
    /// parse token as number of beats
    /// returns length and if staccato
    fn length(&self, token: &str) -> Result<(f64, bool), String> {
        match token.parse::<usize>() {
            // normal note value
            Ok(length) => Ok((1.0 / length as f64, false)),
            Err(..) => match token.as_bytes().last() {
                Some(&DOTTED) => Ok((self.scale(token, 1.5, DOTTED), false)),
                Some(&STACCATO) => Ok((self.scale(token, 0.5, STACCATO), true)),
                _ if self.has_tie(token) => Ok((self.parse_tie(token), false)),
                _ => Err(format!("invalid token as note length: {}", token)),
            }
        }
    }
    /// parse token as frequency
    fn frequency(&mut self, token: &str) -> Result<f64, String> {
        // https://en.wikipedia.org/wiki/Piano_key_frequencies
        let key_num = self.key_number(token)?;
        Ok(*self.notes.entry(key_num).or_insert(
            2f64.powf((key_num as f64 - 49.0) / 12.0) * 440.0
        ))
    }

    /// check if token is a frequency
    fn is_pitch(&self, token: &str) -> bool {
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
        // sum up each value
        token.split(TIE as char).flat_map(|s| s.parse::<f64>()).map(|f| 1.0 / f).sum()
    }
    /// scale the duration of a token as length
    fn scale(&self, token: &str, scale: f64, suffix: u8) -> f64 {
        scale / token.strip_suffix(suffix as char).unwrap().parse::<f64>().unwrap()
    }
    /// convert note to key number
    fn key_number(&self, note: &str) -> Result<usize, String> {
        let (tone, octave) = note.split_at(note.len() - 1);
        match (self.tones.get(tone), octave.parse::<i32>()) {
            (Some(i), Ok(o)) => Ok((i + o * 12) as usize),
            _ => Err(format!("invalid token as note frequency: {}", note)),
        }
    }
}
