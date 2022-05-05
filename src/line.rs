use crate::writer::Writer;

#[derive(Clone)]
pub struct Line {
    pub size: usize,
    pub offset: usize,
    pub notes: Vec<(usize, f64)>
}

impl Line {
    pub fn new() -> Self {
        Self {
            size: 0,
            offset: 0,
            notes: vec![]
        }
    }
}
