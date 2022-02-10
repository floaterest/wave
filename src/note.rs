const TONES: [u8; 11] = [
    'a' as u8,
    0,
    'b' as u8,
    'c' as u8,
    0,
    'd' as u8,
    0,
    'e' as u8,
    'f' as u8,
    0,
    'g' as u8
];

/// convert note to index (key number)
/// e.g. a3 to 36, db3 to 30, f4 to 44 (zero-indexed)
fn ntoi(n: &[u8]) -> u8 {
    let l = n.len();
    let mut i = TONES.iter().position(|&ch| ch == n[0]).unwrap() as u8;
    // if flat or sharp
    if l == 3 {
        // +1 if sharp, -1 if flat
        i = if n[2] == 'b' as u8 { i - 1 } else { i + 1 };
    }
    // https://en.wikipedia.org/wiki/Piano_key_frequencies
    // 48 is '0'
    i + 12 * (n[l - 1] - 48 - (i > 2) as u8)
}

/// convert note to its frequency
pub fn ntof(n: &[u8]) -> f64 {
    // -48 because zero-index
    2f64.powf((ntoi(n) as f64 - 48.0) / 12.0) * 440.0
}