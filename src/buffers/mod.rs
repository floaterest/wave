//! structs that stores notes
pub use capture::CaptureMap;
pub use line::{Chord, Line};
pub use repeat::Repeat;
pub use waveform::Waveform;

mod repeat;
mod waveform;
mod capture;
mod line;

