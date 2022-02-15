use std::fs::File;
use std::slice::from_raw_parts;
use std::mem::transmute;
use std::f64::consts::PI;
use std::io::{Result, Seek, SeekFrom, Write};

pub struct Wave<'a> {
    pub frame_rate: u32,
    /// in bytes
    pub frame_width: u16,
    pub nchannels: u16,
    pub file: File,
    pub amplitude: f64,
    /// curve function
    pub fx: &'a dyn Fn(f64) -> f64,
    /// number of frames that are off beat
    offset: i8,
    inc: bool,
}

impl Wave<'_> {
    pub fn new<'a>(frame_rate: u32, amplitude: f64, fname: &str, fx: &'a dyn Fn(f64) -> f64) -> Wave<'a> {
        Wave {
            frame_rate,
            amplitude,
            fx,
            frame_width: 2,
            nchannels: 1,
            file: File::create(fname).expect("Wave: Create file failed"),
            offset: 0,
            inc: false,
        }
    }

    // write headers
    pub fn start(&mut self) -> Result<()> {
        self.file.write(&[
            82, 73, 70, 70, // RIFF
            0, 0, 0, 0, // file size
            87, 65, 86, 69, // WAVE
            102, 109, 116, 32, // fmt
            16, 0, 0, 0, // fmt chunk size
            1, 0, // format tag (PCM)
        ])?;
        self.file.write(&unsafe { transmute(self.nchannels) } as &[u8; 2])?;
        // frame rate (fps)
        self.file.write(&unsafe { transmute(self.frame_rate) } as &[u8; 4])?;
        // byte rate
        self.file.write(&unsafe { transmute(self.frame_rate * self.frame_width as u32) } as &[u8; 4])?;
        // block align
        self.file.write(&[2, 0])?;
        // bits per frame
        self.file.write(&unsafe { transmute(self.frame_width * 8) } as &[u8; 2])?;

        self.file.write(&[
            100, 97, 116, 97, // data
            0, 0, 0, 0, // nframes * nchannels * bytes / frame, also is file size - 36
        ])?;

        Ok(())
    }

    // write frame data
    pub fn write(&mut self, freqs: &[f64], duration: f32) -> Result<()> {
        let nframes = (duration * self.frame_rate as f32) as u32;
        let n = 2.0 * PI / self.frame_rate as f64;
        let a = if self.inc { self.amplitude } else { -self.amplitude };

        let mut sines: Vec<i16> = (1..nframes).map(|i|
            (a * &(self.fx)(i as f64 / nframes as f64) * freqs.iter().map(|f| (f * n * i as f64).sin()).sum::<f64>()) as i16
        ).collect();
        let pos = sines[sines.len() - 1] > 0;
        self.inc = !pos;
        let lpos = sines.iter().rposition(|s| (s < &0) == pos).unwrap() as i16;
        let mut rpos = nframes as i16;
        loop {
            let sin = (a * &(self.fx)(rpos as f64 / nframes as f64) * freqs.iter().map(|f| (f * n * rpos as f64).sin()).sum::<f64>()) as i16;
            if (sin < 0) == pos {
                break;
            }
            sines.push(sin);
            rpos += 1;
        }
        let beat = nframes as i16 + self.offset as i16;
        // if left is closer than right
        if beat - lpos < rpos - beat {
            self.file.write(unsafe {
                from_raw_parts(sines.as_ptr() as *const u8, (lpos as usize + 1) * 2)
            })?;
            self.offset = (lpos - nframes as i16) as i8;
            self.inc = !self.inc;
        } else {
            self.file.write(unsafe {
                from_raw_parts(sines.as_ptr() as *const u8, sines.len() * 2)
            })?;
            self.offset = (rpos - nframes as i16) as i8;
        }

        Ok(())
    }

    /// go back and write file size
    pub fn finish(&mut self) -> Result<()> {
        let sz: u64 = self.file.metadata()?.len();
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write(&unsafe { transmute(sz as u32) } as &[u8; 4])?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write(&unsafe { transmute((sz - 36) as u32) } as &[u8; 4])?;

        Ok(())
    }
}
