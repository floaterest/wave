use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::slice::from_raw_parts;
use std::mem::{size_of, transmute};
use std::io::{Result, Seek, SeekFrom, Write};
use crate::note::ntof;
use crate::{DOTTED, Repeat, STACCATO, TIE};
use crate::curves::sinusoid;

fn sine(ampl: f64, i: f64, n: f64, period: f64, curve: &dyn Fn(f64) -> f64) -> i16 {
    (ampl * curve(i / n) * (period * i).sin()) as i16
}

pub struct Wave {
    /// output file (`.wav`)
    file: File,
    /// number of samples/frames per second (fps)
    rate: u32,
    /// maximum amplitude of a note
    amplitude: f64,

    /// current bpm
    bpm: u16,
    /// current position
    pos: u64,
    /// waveform buffer
    buffer: Vec<i16>,
    /// PI * 2.0 * frame rate
    pi2rate: f64,
    /// note -> freq
    notes: HashMap<String, f64>,
}

impl Wave {
    pub fn new(destination: File, rate: u32, amplitude: f64) -> Self {
        Wave {
            file: destination,
            rate,
            amplitude,

            bpm: 0,
            pos: 0,
            buffer: Vec::new(),
            pi2rate: 2.0 * PI / rate as f64,
            notes: HashMap::new(),
        }
    }
    /// resize the buffer and fill with 0
    pub fn resize(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }
    //#region write to file
    /// write headers
    pub fn start(&mut self) -> Result<()> {
        let nchannels = 1u16;
        let frame_width = 16u16;
        self.file.write(&[
            82, 73, 70, 70, // RIFF
            0, 0, 0, 0, // file size
            87, 65, 86, 69, // WAVE
            102, 109, 116, 32, // fmt
            16, 0, 0, 0, // fmt chunk size
            1, 0, // format tag (PCM)
        ])?;
        self.file.write(&unsafe { transmute(nchannels) } as &[u8; 2])?;
        // frame rate (fps)
        self.file.write(&unsafe { transmute(self.rate) } as &[u8; 4])?;
        // byte rate
        self.file.write(&unsafe { transmute(self.rate * frame_width as u32) } as &[u8; 4])?;
        // block align
        self.file.write(&[2, 0])?;
        // bits per frame
        self.file.write(&unsafe { transmute(frame_width) } as &[u8; 2])?;

        self.file.write(&[
            100, 97, 116, 97, // data
            0, 0, 0, 0, // nframes * nchannels * bytes / frame, also is file size - 36
        ])?;

        Ok(())
    }
    /// add a note to existing waveform (buffer)
    pub fn append(&mut self, len: usize, freq: f64) {
        // no need to add rests
        if freq == 0.0 { return; }
        assert_ne!(len, 0, "Frame count is 0 at {}!", freq);
        assert_ne!(self.bpm, 0, "BPM is 0.0 at {}!", freq);
        let a = self.amplitude;
        let period = freq * self.pi2rate;
        // let frames = (0..len).map(|i| i as f64)
        (0..len).map(|i| i as f64)
            .map(|i| sine(a, i, len as f64, period, &sinusoid))
            .enumerate().for_each(|(i, y)| self.buffer[i] += y);
    }
    /// write frames and shift position
    pub fn flush(&mut self, offset: usize) -> Result<()> {
        self.pos += offset as u64;
        let wave: Vec<_> = self.buffer.drain(..offset).collect();
        self.file.write(unsafe { from_raw_parts(wave.as_ptr() as *const u8, wave.len() * size_of::<i16>()) })?;
        Ok(())
    }
    /// go back and write file size
    pub fn finish(&mut self) -> Result<()> {
        // empty buffer
        self.flush(self.buffer.len())?;

        let size: u64 = self.file.metadata()?.len();
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write(&unsafe { transmute(size as u32) } as &[u8; 4])?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write(&unsafe { transmute((size - 36) as u32) } as &[u8; 4])?;

        Ok(())
    }
    //#endregion write to file
    //#region parse input
    /// process a line (notes/bpm) of input
    pub fn process(&mut self, line: &str, repeat: &mut Repeat) -> Result<()> {
        // if the line specifies the bpm
        Ok(match line.parse::<u16>() {
            Ok(bpm) => self.bpm = bpm,
            Err(..) => self.parse_notes(line, repeat)?,
        })
    }
    /// parse a line as notes
    fn parse_notes(&mut self, line: &str, repeat: &mut Repeat) -> Result<()> {
        let mut offset = 0;
        let mut len = 0;
        let mut size = self.buffer.len();
        line.split_ascii_whitespace().for_each(|token| match token.bytes().next() {
            // if is note length
            Some(b) if b.is_ascii_digit() => {
                len = self.parse_len(token);
                if len > size {
                    size = len;
                    self.buffer.resize(size, 0);
                }
                // always take shortest len
                if offset == 0 || len < offset { offset = len; }
                // half note length of staccato
                if token.ends_with(STACCATO) { len /= 2; }
            }
            _ => {
                // parse token as note
                if !self.notes.contains_key(token) {
                    self.notes.insert(token.to_string(), ntof(token.as_bytes()));
                }
                let freq = *self.notes.get(token).unwrap();
                self.append(len, freq);
                repeat.push(len, freq);
            }
        });
        repeat.resize(size, offset);
        repeat.flush();
        Ok(self.flush(offset)?)
    }
    /// parse a token as length of a note and return number of frames
    fn parse_len(&self, token: &str) -> usize {
        // assume first character is ascii digit
        let length = match token.len() {
            // e.g. "8" for quaver
            _ if token.bytes().all(|b| b.is_ascii_digit()) => 1.0 / token.parse::<f64>().unwrap(),
            // e.g. "4." for dotted crotchet
            2 if token.ends_with(DOTTED) => 1.5 / token.strip_suffix(DOTTED).unwrap().parse::<f64>().unwrap(),
            // e.g. "8*" for quaver with staccato
            2 if token.ends_with(STACCATO) => 1.0 / token.strip_suffix(STACCATO).unwrap().parse::<f64>().unwrap(),
            // e.g. "8+16" for a tie from quaver to semiquaver
            3 if token.chars().all(|ch| ch.is_ascii_digit() || ch == TIE) => token.split(TIE)
                .map(|s| 1.0 / s.parse::<f64>().unwrap())
                .sum(),
            _ => {
                assert!(false, "Unknown token as note length: {:?}", token);
                0.0
            },
        };
        //     ((      duration in seconds       )  number of frames)
        return ((length * 240.0 / self.bpm as f64) * self.rate as f64) as usize;
    }
    //#endregion parse input
}
