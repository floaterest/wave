use std::collections::HashSet;
use std::rc::Rc;

use crate::buffers::Waveform;
use crate::line::Line;
use crate::writer::Writer;

pub struct Repeat {
    pub voltas: Vec<Vec<Rc<Line>>>,
    pub current: usize,
    pub to_store: HashSet<usize>
}

impl Repeat {
    pub fn new() -> Self {
        Self { voltas: Vec::new(), current: 1, to_store: HashSet::new() }
    }
    // update set of voltas to store
    pub fn start(&mut self, indices: &[usize]) {
        match indices.iter().max() {
            Some(&size) => {
                let size = size + 1;
                if size > self.voltas.len() { self.voltas.resize(size, Vec::new()) }
                // don't store current volta
                self.to_store = indices.iter().filter(|&&i| i != self.current).cloned().collect();
            },
            None => panic!("Want to start repeat but indices are empty!")
        }
    }
    /// add a new note to the current voltas
    pub fn push(&mut self, line: Line) {
        let rc = Rc::new(line);
        for &v in self.to_store.iter() {
            self.voltas[v].push(rc.clone());
        }
    }
    /// free all data
    pub fn clear(&mut self) {
        self.to_store.clear();
        self.voltas.clear();
        self.current = 1;
    }

    //#region write
    /// write a volta
    fn write(&mut self, v: usize, buffer: &mut Waveform, write: &mut dyn FnMut(Vec<i16>)) {
        for line in self.voltas[v].iter() {
            buffer.fold_with_line(line);
            write(buffer.drain_until(line.offset()));
        }
    }
    /// repeat all needed voltas
    pub fn repeat(&mut self, buffer: &mut Waveform, writer: &mut Writer) {
        let mut write = |data| writer.write(data).unwrap();
        // append repeat
        self.write(0, buffer, &mut write);
        // ready to store next volta
        self.current += 1;
        // if the next volta is already stored (∴ won't appear in input)
        if self.voltas.len() > self.current && self.voltas[self.current].len() > 0 {
            // append current volta
            self.write(self.current, buffer, &mut write);
            // append repeat again
            self.write(0, buffer, &mut write);
            self.current += 1;
        }
    }
    //#endregion write
}
