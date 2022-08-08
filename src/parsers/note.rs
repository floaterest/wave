use std::collections::HashMap;

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
    /// convert note to key number
    fn key_number(&self, note: &str) -> usize {
        let (tone, octave) = note.split_at(note.len() - 1);
        match (self.tones.get(tone), octave.parse::<i32>()) {
            (Some(i), Ok(oct)) => (i + oct * 12) as usize,
            _ => panic!("Invalid token as note: {}", note),
        }
    }
    /// convect note to frequency, returns 0.0 if note is rest
    pub fn frequency(&mut self, note: &str) -> f64 {
        if note.len() == 1 && !note.as_bytes()[0].is_ascii_alphanumeric() {
            // rest
            0.0
        } else {
            // https://en.wikipedia.org/wiki/Piano_key_frequencies
            let kn = self.key_number(note);
            *self.notes.entry(kn).or_insert(2f64.powf((kn as f64 - 49.0) / 12.0) * 440.0)
        }
    }
}
