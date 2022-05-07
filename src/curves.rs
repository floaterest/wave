use std::f64::consts::PI;

/// make rectangle shape
// pub fn constant(_: f64) -> f64 { 1.0 }

/// make sine shape
pub fn sinusoid(x: f64) -> f64 { ((x * PI).cos() + 1.0) / 2.0 }


/// create waveform
pub fn sine(i: f64, n: f64, period: f64, curve: &dyn Fn(f64) -> f64) -> f64 {
    curve(i / n) * (period * i).sin()
}
