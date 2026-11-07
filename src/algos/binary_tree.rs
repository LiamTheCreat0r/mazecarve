//! Binary tree: carve north or east at every cell; diagonal bias with
//! two clear open edges.

use super::Algo;
use crate::maze::{DIR_E, DIR_N, Maze};
use crate::rng::Rng;

pub struct BinaryTree {
    row: i32,
    col: i32,
}

pub fn create(m: &mut Maze, _rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(true);
    Box::new(BinaryTree { row: 0, col: 0 })
}

impl Algo for BinaryTree {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        m.mark(self.row, self.col);
        m.set_head(self.row, self.col);

        let mut dirs = [0usize; 2];
        let mut n = 0;
        if self.row > 0 {
            dirs[n] = DIR_N;
            n += 1;
        }
        if self.col < m.cols - 1 {
            dirs[n] = DIR_E;
            n += 1;
        }
        if n > 0 {
            // Only the top-right cell has no choice.
            let d = dirs[rng.below(n as u32) as usize];
            m.carve(self.row, self.col, d);
        }

        self.col += 1;
        if self.col >= m.cols {
            self.col = 0;
            self.row += 1;
        }
        if self.row >= m.rows {
            m.set_head(-1, -1);
            return true;
        }
        false
    }
}
