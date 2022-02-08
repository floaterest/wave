use std::fs::File;
use std::mem::transmute;
use std::f64::consts::PI;
use std::io::{Result, Seek, SeekFrom, Write};

pub struct Wave {
    pub frame_rate: u32,
}

impl Wave {
    pub fn new(fr: u32) -> Wave {
        Wave {
            frame_rate: fr,
        }
    }

    pub fn write(&self, freq: f64, ampl: f64, d: u32, filename: String) -> Result<()> {
        let nchannels = 1i16;
        let bpf = 2u32;
        let fw: u16 = (bpf * 8) as u16;

        let mut f = File::create(filename)?;

        //#region header
        f.write(&[
            82, 73, 70, 70, // RIFF
            0, 0, 0, 0, // file size
            87, 65, 86, 69, // WAVE
            102, 109, 116, 32, // fmt
            16, 0, 0, 0, // fmt chunk size
            1, 0, // format tag (PCM)
        ])?;
        f.write(&unsafe { transmute(nchannels) } as &[u8; 2])?;
        // frame rate (fps)
        f.write(&unsafe { transmute(self.frame_rate) } as &[u8; 4])?;
        // byte rate
        f.write(&unsafe { transmute(self.frame_rate * bpf) } as &[u8; 4])?;
        // block align
        f.write(&[2, 0])?;
        // frame width (bits per frame)
        f.write(&unsafe { transmute(fw) } as &[u8; 2])?;

        f.write(&[
            100, 97, 116, 97, // data
            0, 0, 0, 0, // nframes * nchannels * bytes / frame, also is file size - 36
        ])?;
        //#endregion header

        //#region data
        let nframes = d * self.frame_rate;
        let n = 2.0 * PI * freq / self.frame_rate as f64;
        let mut frames: Vec<u8> = Vec::with_capacity((nframes * bpf) as usize);
        (0..nframes).for_each(|i| {
            frames.extend_from_slice(&unsafe { transmute((ampl * (n * i as f64).sin()) as i16) } as &[u8; 2])
        });
        f.write(&frames)?;
        //#endregion data

        //#region go back and write file size
        let sz: u64 = f.metadata()?.len();
        f.seek(SeekFrom::Start(4))?;
        f.write(&unsafe { transmute(sz as u32) } as &[u8; 4])?;
        f.seek(SeekFrom::Start(40))?;
        f.write(&unsafe { transmute((sz - 36) as u32) } as &[u8; 4])?;
        //#endregion go back and write file size
        Ok(())
    }
}
