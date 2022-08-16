// use std::cell::RefCell;
// use std::collections::{HashMap, HashSet};
// use std::rc::Rc;
//
// use crate::stores::note::Line;
//
// pub struct Repeat {
//     /// 0 for pre-volta, MAX for post-volta
//     voltas: HashMap<usize, Rc<RefCell<Vec<Line>>>>,
//     /// indices of one of the voltas to record
//     current: usize,
// }
//
// impl Repeat {
//     pub fn new() -> Self {
//         Self {
//             voltas: HashMap::new(),
//             current: 0,
//         }
//     }
//     /// return if Repeat is currently recording
//     pub fn on_rec(&self) -> bool {
//         self.voltas.is_empty()
//     }
//     /// init new voltas to store
//     pub fn start(&mut self, indices: &[usize]) {
//         let volta = Rc::new(RefCell::new(Vec::new()));
//         for &i in indices.iter() {
//             self.voltas.insert(i, Rc::clone(&volta));
//         }
//     }
//     /// add new line to current voltas
//     pub fn push(&mut self, line: Line) {
//         match self.voltas.get(&self.current) {
//             Some(volta) => volta.borrow_mut().push(line),
//             None => if self.current == usize::MAX {
//                 panic!("Volta No. MAX is not initialised while trying to push")
//             } else {
//                 panic!("Volta No. {} is not initialised while trying to push", self.current)
//             }
//         }
//     }
//     /// reset everything
//     pub fn clear(&mut self) {
//         self.voltas.clear();
//         self.current = 0;
//     }
// }