use std::f64::consts::PI;
use std::fs::File;
use std::slice::from_raw_parts;
use std::mem::{size_of, transmute};
use std::io::{Result, Seek, SeekFrom, Write};
use crate::note::ntof;

pub struct Wave<'a> {
    /// output file (`.wav`)
    file: File,
    /// number of samples/frames per second (fps)
    fps: u32,
    /// maximum amplitude of a note
    amplitude: f64,
    /// curve function in [0.0, 1.0)
    curve: &'a dyn Fn(f64) -> f64,

    /// current bpm
    bpm: i32,
    /// current position
    pos: u64,
    /// waveform buffer
    buffer: Vec<i16>,
    /// PI * 2.0 * frame rate
    pi2_fps: f64,
}

impl Wave<'_> {
    pub fn new<'a>(destination: File, fps: u32, amplitude: f64, curve: &'a dyn Fn(f64) -> f64) -> Wave<'a> {
        Wave {
            file: destination,
            fps,
            amplitude,
            curve,

            bpm: 0,
            pos: 0,
            buffer: Vec::new(),
            pi2_fps: 2.0 * PI / fps as f64,
        }
    }

    /// parse the length of a note and return number of frames
    fn parse_len(&self, token: &str) -> usize {
        let length = match token.len() {
            // e.g. "8" for quaver
            _ if token.bytes().all(|b| b.is_ascii_digit()) => 1.0 / token.parse::<f64>().unwrap(),
            // e.g. "4*" for dotted crotchet
            2 if token.ends_with('*') => 1.5 / token.strip_suffix('*').unwrap().parse::<f64>().unwrap(),
            // e.g. "8+16" for a tie from quaver to semiquaver
            3 if token.bytes().all(|ch| ch.is_ascii_digit() || ch == b'+') => token.bytes()
                .filter(|&b| b.is_ascii_digit())
                .map(|b| 1.0 / (b - b'0') as f64).sum(),
            _ => {
                assert!(false, "Unknown token as note length: {:?}", token);
                0.0
            },
        };
        //     ((   duration in seconds   )  number of frames)
        return ((length * 240.0 / self.bpm as f64) * self.fps as f64) as usize;
    }
    /// generate sine value (y) at x
    fn sine(&self, a: f64, x: f64, n: f64, freq: f64, curve: &dyn Fn(f64) -> f64) -> i16 {
        (a * curve(x / n) * (freq * self.pi2_fps * x).sin()) as i16
    }

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
        self.file.write(&unsafe { transmute(self.fps) } as &[u8; 4])?;
        // byte rate
        self.file.write(&unsafe { transmute(self.fps * frame_width as u32) } as &[u8; 4])?;
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
    fn append(&mut self, frame_count: usize, note: &str) {
        assert_ne!(frame_count, 0, "Frame count is 0 at {}!", note);
        assert_ne!(self.bpm, 0, "BPM is 0.0 at {}!", note);
        let freq = ntof(note.as_bytes());
        // negative amplitude will make wave decrease on start
        // let amplitude = if self.inc { self.amplitude } else { -self.amplitude };
        let amplitude = self.amplitude;
        let frames = (0..frame_count).map(|i| i as f64)
            .map(|i| self.sine(amplitude, i, frame_count as f64, freq, self.curve))
            .collect::<Vec<_>>();
        frames.iter().enumerate().for_each(|(i, y)| self.buffer[i] += y);
        // for (i, &y) in frames.iter().enumerate() {
        //     self.buffer[i] += y;
        // }
    }
    /// process a line of input
    pub fn process(&mut self, line: &[&str]) -> Result<()> {
        // if the line specifies the bpm
        if line.len() == 1 && line[0].bytes().all(|b| b.is_ascii_digit()) {
            self.bpm = line[0].parse().unwrap();
        } else {
            let mut offset = 0;
            let mut frame_count = 0;
            line.iter().for_each(
                // if is note length
                |token| if token.bytes().next().unwrap().is_ascii_digit() {
                    // if this length is the first of the line
                    if frame_count == 0 {
                        offset = self.parse_len(token);
                        frame_count = offset;
                    } else {
                        frame_count = self.parse_len(token);
                    }
                } else { // parse token as note
                    // len (in beats) == beat * 4
                    //      semibreve == 1 == 1 beats
                    //      quaver == 0.125 == 0.5 beats
                    // dur (in seconds) = len * (60 / bpm) = len * (60 * second / beat)
                    if frame_count > self.buffer.len() {
                        self.buffer.resize(frame_count, 0);
                    }
                    self.append(frame_count, token);
                }
            );
            return Ok(self.flush(offset)?)
        }
        Ok(())
    }
    /// write frames and shift position
    fn flush(&mut self, frame_count: usize) -> Result<()> {
        self.pos += frame_count as u64;
        let wave: Vec<_> = self.buffer.drain(..frame_count).collect();
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
}
