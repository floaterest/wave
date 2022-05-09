# Wave
> Generate `.wav` file from user input

## Usage
- from binary: `wave [input] [output]`
- from source code: `cargo r [input] [output]`

### Command Line Arguments

- `<input>`: input text file, `input.txt` by default
- `<output>`: output wav file, `output.wav` by default

## Input Format

see [input.md](./doc/input.md)

# Programmer's Note


- [writer.rs](./src/writer.rs)
  - write `.wav` file headers
  - get file size from metadata
- [stores/waveform.rs](./src/stores/waveform.rs)
  - generate waveform for `.wav` given frequency and frame count
- [parsers/capture.rs](./src/parsers/capture.rs)
  - use `Rc<T>` to avoid duplicate data stored in heap
- [parsers/input.rs](./src/parsers/input.rs)
  - use `Peekable<T>` for token lookaheads
- [parsers/note.rs](./src/parsers/note.rs)
  - convert pitch in scientific notation to it's frequency in `O(1)` time
  - use `HashMap<K,V>` to cache the frequencies
- [parsers/repeat.rs](./src/parsers/repeat.rs)
  - use `Rc<RefCell<T>>` to avoid duplicate and allow mutable reference
  - use `BTreeMap<K,V>` for ordered map
  - higher order functions

# Todo

- [] find a shorter example of tie