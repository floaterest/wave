use std::f64::consts::PI;

// pub fn constant(_: f64) -> f64 { 1.0 }

pub fn sinusoid(x: f64) -> f64 { ((x * PI).cos() + 1.0) / 2.0 }
