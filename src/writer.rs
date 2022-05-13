use std::fs::File;
use std::io::{Result, Seek, SeekFrom, Write};
use std::mem::{size_of, transmute};
use std::slice::from_raw_parts;

pub struct Writer {
    /// output file (`.wav`)
    file: File,
}

impl Writer {
    pub fn new(destination: File) -> Self {
        Self { file: destination }
    }
    /// write headers
    pub fn start(&mut self, rate: u32) -> Result<()> {
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
        self.file.write(&unsafe { transmute(rate) } as &[u8; 4])?;
        // byte rate
        self.file.write(&unsafe { transmute(rate * frame_width as u32) } as &[u8; 4])?;
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
    /// write frames and shift position
    pub fn write(&mut self, waveform: Vec<i16>) -> Result<()> {
        self.file.write(unsafe {
            from_raw_parts(waveform.as_ptr() as *const u8, waveform.len() * size_of::<i16>())
        })?;
        Ok(())
    }
    /// go back and write file size
    pub fn finish(&mut self) -> Result<()> {
        let size: u64 = self.file.metadata()?.len();
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write(&unsafe { transmute(size as u32) } as &[u8; 4])?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write(&unsafe { transmute((size - 36) as u32) } as &[u8; 4])?;

        Ok(())
    }
}
