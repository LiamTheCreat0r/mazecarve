//! Recursive backtracker (depth first search): long, winding corridors
//! with few branches.

use super::Algo;
use crate::maze::Maze;
use crate::rng::Rng;

pub struct Backtracker {
    stack: Vec<usize>,
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(true);
    let r = rng.below(m.rows as u32) as i32;
    let c = rng.below(m.cols as u32) as i32;
    m.mark(r, c);
    m.set_head(r, c);
    Box::new(Backtracker {
        stack: vec![m.idx(r, c)],
    })
}

impl Algo for Backtracker {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        let Some(&cur) = self.stack.last() else {
            return true;
        };
        let r = cur as i32 / m.cols;
        let c = cur as i32 % m.cols;

        let mut dirs = [0usize; 4];
        let mut n = 0;
        for d in 0..4 {
            if let Some((nr, nc)) = m.nb(r, c, d)
                && !m.vis[m.idx(nr, nc)]
            {
                dirs[n] = d;
                n += 1;
            }
        }

        if n == 0 {
            // Dead end: backtrack.
            self.stack.pop();
            match self.stack.last() {
                Some(&p) => m.set_head(p as i32 / m.cols, p as i32 % m.cols),
                None => m.set_head(-1, -1),
            }
            return self.stack.is_empty();
        }

        let d = dirs[rng.below(n as u32) as usize];
        m.carve(r, c, d);
        let (nr, nc) = m.nb(r, c, d).expect("checked above");
        m.mark(nr, nc);
        m.set_head(nr, nc);
        self.stack.push(m.idx(nr, nc));
        false
    }
}
