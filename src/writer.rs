use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::slice::from_raw_parts;
use std::mem::{size_of, transmute};
use std::io::{Result, Seek, SeekFrom, Write};
use crate::note::ntof;
use crate::{DOTTED, Repeat, STACCATO, TIE};
use crate::curves::sinusoid;
use crate::line::Line;

fn sine(ampl: f64, i: f64, n: f64, period: f64, curve: &dyn Fn(f64) -> f64) -> i16 {
    (ampl * curve(i / n) * (period * i).sin()) as i16
}

pub struct Writer {
    /// output file (`.wav`)
    file: File,
    /// number of samples/frames per second (fps)
    pub rate: u32,
    /// maximum amplitude of a note
    amplitude: f64,

    /// current bpm
    pub bpm: u16,
    /// current position
    pos: u64,
    /// waveform buffer
    pub buffer: Vec<i16>,
    /// PI * 2.0 * frame rate
    pi2rate: f64,
}

impl Writer {
    pub fn new(destination: File, rate: u32, amplitude: f64) -> Self {
        Self {
            file: destination,
            rate,
            amplitude,

            bpm: 0,
            pos: 0,
            buffer: Vec::new(),
            pi2rate: 2.0 * PI / rate as f64,
        }
    }
    /// resize the buffer and fill with 0
    pub fn resize(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }
    //#region write to file
    /// write headers
    pub fn start(&mut self) -> Result<()> {
        let nch = 1u16;
        let frame_width = 16u16;
        self.file.write(&[
            82, 73, 70, 70, // RIFF
            0, 0, 0, 0, // file size
            87, 65, 86, 69, // WAVE
            102, 109, 116, 32, // fmt
            16, 0, 0, 0, // fmt chunk size
            1, 0, // format tag (PCM)
        ])?;
        self.file.write(&unsafe { transmute(nch) } as &[u8; 2])?;
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
    pub fn append_line(&mut self, line: &Line) {
        if line.size == 0 { return; }
        self.resize(line.size);
        line.notes.iter().for_each(|(n, freq)| self.append(*n, *freq));
        self.flush(line.offset).unwrap();
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
}
