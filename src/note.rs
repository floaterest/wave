const TONES: [char; 11] = ['a', '\0', 'b', 'c', '\0', 'd', '\0', 'e', 'f', '\0', 'g'];

/// convert note to index (key number)
/// e.g. a3 to 36, db3 to 30, f4 to 44 (zero-indexed)
fn ntoi(n: &[char]) -> usize {
    let l = n.len();
    let mut i = TONES.iter().position(|&ch| ch == n[0]).unwrap();
    // if flat or sharp
    if l == 3 {
        // +1 if sharp, -1 if flat
        i = if n[2] == 'b' { i - 1 } else { i + 1 };
    }
    // https://en.wikipedia.org/wiki/Piano_key_frequencies
    // 48 is '0'
    i + 12 * (n[l - 1] as usize - 48 - (i > 2) as usize)
}

/// convert note to its frequency
pub fn ntof(n: &[char]) -> f64 {
    // -48 because zero-index
    2f64.powf((ntoi(n) as f64 - 48.0) / 12.0) * 440.0
}