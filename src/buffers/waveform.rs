use std::f64::consts::PI;

use crate::curves::{sine, sinusoid};
use crate::line::Line;

pub struct Waveform {
    /// current bpm
    pub bpm: u16,
    /// maximum amplitude of a note
    pub amp: f64,
    /// number of samples/frames per second
    pub fps: u32,
    /// waveform buffer
    buffer: Vec<i16>,
}

impl Waveform {
    pub fn new(amp: f64, fps: u32) -> Self {
        Self { amp, bpm: 0, fps, buffer: Vec::new() }
    }
    pub fn drain_to(&mut self, end: usize) -> Vec<i16> {
        self.buffer.drain(..end).collect()
    }
    pub fn drain_all(&mut self) -> Vec<i16> {
        self.buffer.drain(..).collect()
    }
    /// add a note to waveform
    pub fn add(&mut self, len: usize, freq: f64) {
        // no need to add rests
        if freq == 0.0 { return; }
        assert_ne!(len, 0, "Frame count is 0 at {} Hz", freq);
        assert_ne!(self.bpm, 0, "BPM is 0.0 at {} Hz", freq);
        let period = freq * PI * 2.0 / self.fps as f64;
        let a = self.amp;
        // add new wave to buffer
        (0..len).map(
            |i| a * sine(i as f64, len as f64, period, &sinusoid)
        ).enumerate().for_each(|(i, y)| self.buffer[i] += y as i16)
    }
    // add a line to waveform
    pub fn add_line(&mut self, line: &Line) {
        let size = line.size();
        assert_ne!(size, 0, "Line size is 0 when trying to generate waveform!");
        if self.buffer.len() < size {
            self.buffer.resize(size, 0);
        }
        for chord in line.chords() {
            for freq in chord.frequencies.iter() {
                self.add(chord.length, *freq);
            }
        }
    }
}