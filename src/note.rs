const TONES: [u8; 12] = [
    b'c',
    0,
    b'd',
    0,
    b'e',
    b'f',
    0,
    b'g',
    0,
    b'a',
    0,
    b'b',
];

/// convert note to index (key number)
/// e.g. a3 to 36, db3 to 30, f4 to 44 (zero-indexed)
fn ntoi(n: &[u8]) -> u8 {
    let l = n.len();
    let mut i = TONES.iter().position(|&ch| ch == n[0]).unwrap() as u8;
    // if flat or sharp
    if l == 3 {
        // +1 if sharp, -1 if flat
        i = if n[1] == 'b' as u8 { i - 1 } else { i + 1 };
    }
    // https://en.wikipedia.org/wiki/Piano_key_frequencies
    // 48 is '0', so -49 for -1
    i + 4 + 12 * (n[l - 1] - 49)
}

/// convert note to its frequency
pub fn ntof(n: &[u8]) -> f64 {
    if n[0].is_ascii_alphabetic() {
        // https://en.wikipedia.org/wiki/Piano_key_frequencies
        2f64.powf((ntoi(n) as f64 - 49.0) / 12.0) * 440.0
    } else {
        0.0 // rest
    }
}