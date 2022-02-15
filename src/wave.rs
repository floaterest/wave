use std::fs::File;
use std::slice::from_raw_parts;
use std::mem::transmute;
use std::io::{Result, Seek, SeekFrom, Write};

pub struct Wave<'a> {
    // number of samples/frames per second (fps)
    pub frame_rate: u32,
    // maximum amplitude of a note
    pub amplitude: f64,
    /// curve function
    pub fx: &'a dyn Fn(f64) -> f64,

    pub file: File,
    /// in bytes
    pub frame_width: u16,
    /// number of channels
    pub nchannels: u16,

    /// number of frames that are off beat
    offset: i8,
    /// whether the next wave should start increasing
    inc: bool,
    /// PI * 2.0 * frame rate
    pi2_fps: f64,
}

impl Wave<'_> {
    pub fn new<'a>(frame_rate: u32, amplitude: f64, fname: &str, fx: &'a dyn Fn(f64) -> f64) -> Wave<'a> {
        Wave {
            frame_rate,
            amplitude,
            fx,

            file: File::create(fname).expect("Wave: Create file failed"),
            frame_width: 2,
            nchannels: 1,

            offset: 0,
            inc: false,
            pi2_fps: 2.0 * std::f64::consts::PI / frame_rate as f64,
        }
    }

    /// write headers
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

    fn calc(&self, &a: &f64, &x: &f64, &n: &f64, freqs: &[f64]) -> i16 {
        (a * &(self.fx)(x / n) * freqs
            .iter()
            .map(|f| (f * self.pi2_fps * x).sin())
            .sum::<f64>()
        ) as i16
    }

    /// write frame data
    pub fn write(&mut self, freqs: &[f64], duration: f32) -> Result<()> {
        let nframes = (duration * self.frame_rate as f32) as u32;
        // negative amplitude will make the wave start decreasing at start
        let a = if self.inc { self.amplitude } else { -self.amplitude };
        let mut sines: Vec<i16> = (1..nframes)
            .map(|i| self.calc(&a, &(i as f64), &(nframes as f64), freqs))
            .collect();
        // next wave should increase if current wave ends below 0
        self.inc = sines[sines.len() - 1] < 0;
        // find the left position where the wave passed 0
        let lpos = sines.iter().rposition(|s| (s > &0) == self.inc).unwrap() as i16;
        // find the right position by prolonging the wave
        let mut rpos = nframes as i16;
        loop {
            let sin = self.calc(&a, &(rpos as f64), &(nframes as f64), freqs);
            // stop if the wave passed 0
            if (sin > 0) == self.inc { break; }

            sines.push(sin);
            rpos += 1;
        }
        // the position where the wave should end at
        let pos = nframes as i16 + self.offset as i16;
        // if left is closer than right
        if pos - lpos < rpos - pos {
            // write shortened wave and update offset
            self.file.write(unsafe {
                from_raw_parts(sines.as_ptr() as *const u8, (lpos as usize + 1) * 2)
            })?;
            self.offset = (lpos - nframes as i16) as i8;
            // when shortened, the last sample's position will switch from top/bottom to bottom/top
            self.inc = !self.inc;
        } else {
            // write prolonged wave and update offset
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
