use std::f64::consts::PI;

use crate::stores::note::Line;

/// make sine shape
fn sinusoid(x: f64) -> f64 { ((x * PI).cos() + 1.0) / 2.0 }

/// create waveform
fn sine(i: f64, n: f64, period: f64, curve: &dyn Fn(f64) -> f64) -> f64 {
    curve(i / n) * (period * i).sin()
}

pub struct Waveform {
    /// current bpm
    pub bpm: u16,
    /// maximum number of simultaneously playable notes without scaling down the amplitude
    pub max: usize,
    /// number of samples/frames per second
    pub fps: u32,
    /// waveform buffer
    buffer: Vec<i16>,
}

impl Waveform {
    pub fn new(max: usize, fps: u32) -> Self {
        Self { max, bpm: 0, fps, buffer: Vec::new() }
    }
    /// return number of frames given the length as beat
    pub fn frame_count(&self, beat: f64) -> usize {
        assert_ne!(self.bpm, 0, "BPM is at 0.0 while trying to get frame count");
        //      duration in seconds     ) number of frames )
        ((beat * 240.0 / self.bpm as f64) * self.fps as f64) as usize
    }
    //#region fold buffer
    /// add a note onto the waveform
    fn fold_with_note(&mut self, len: usize, freq: f64, max: usize) -> Result<(), String> {
        // no need to add rests
        if freq == 0.0 { return Ok(()); }
        if len == 0 || self.bpm == 0 {
            match (len, self.bpm) {
                (0, 0) => Err(format!("frame count and BPM are 0.0 at {:.2} Hz", freq)),
                (0, _) => Err(format!("frame count is 0 at {:.2} Hz", freq)),
                (_, 0) => Err(format!("BPM is 0 at {:.2} Hz", freq)),
                _ => panic!("wot"),
            }
        } else {
            let period = freq * PI * 2.0 / self.fps as f64;
            let amp = i16::MAX as f64 / self.max.max(max) as f64;
            // add new wave to buffer
            Ok((0..len).map(
                |i| amp * sine(i as f64, len as f64, period, &sinusoid)
            ).enumerate().for_each(|(i, y)| self.buffer[i] += y as i16))
        }
    }
    // fold a new line into the accumulative buffer
    pub fn fold_with_line(&mut self, line: &Line) -> Result<(), String> {
        let size = line.size();
        if size == 0 {
            Err(format!("line size is 0 while trying to add to waveform"))
        } else {
            // resize buffer if needed
            if self.buffer.len() < size {
                self.buffer.resize(size, 0);
            }
            // maximum number of notes to be play at the same time
            let max = line.chords().fold(0, |acc, chord| if chord.count() > acc {
                chord.count()
            } else { acc });

            for chord in line.chords() {
                let length = chord.length;
                for freq in chord.frequencies.iter() {
                    self.fold_with_note(length, *freq, max)?;
                }
            }
            Ok(())
        }
    }
    //#endregion write to buffer
    pub fn drain(&mut self, end: usize) -> Vec<i16> {
        self.buffer.drain(..end).collect()
    }
}