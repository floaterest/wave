# Wave
> Generate `.wav` file from user input

# Usage
- from executable: `wave input.txt output.wav`
- from source: `cargo run input.txt output.wav`

# Programmer's Note
- [`note.rs`](/src/note.rs)
    - convert note to key number to frequency
- [`wave.rs`](/src/wave.rs)
    - `.wav` file structure
    - generate sine wave
    - convert literally anything into bytes (`[u8]`) using unsafe `transmute()`