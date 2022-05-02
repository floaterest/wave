use crate::Wave;

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
    pub fn append(&self, to: &mut Wave) {
        if self.size == 0 { return; }
        to.resize(self.size);
        self.notes.iter().for_each(|(n, freq)| to.append(*n, *freq));
        to.flush(self.offset).unwrap();
    }
}
