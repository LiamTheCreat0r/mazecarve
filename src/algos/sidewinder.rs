//! Sidewinder: horizontal runs, each closed by carving north out of a
//! random member; strong eastward bias.

use super::Algo;
use crate::maze::{DIR_E, DIR_N, Maze};
use crate::rng::Rng;

pub struct Sidewinder {
    run: Vec<i32>,
    row: i32,
    col: i32,
}

pub fn create(m: &mut Maze, _rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(true);
    Box::new(Sidewinder {
        run: Vec::with_capacity(m.cols as usize),
        row: 0,
        col: 0,
    })
}

impl Algo for Sidewinder {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        m.mark(self.row, self.col);
        m.set_head(self.row, self.col);
        if (self.run.len() as i32) < m.cols {
            self.run.push(self.col);
        }

        let at_east = self.col == m.cols - 1;
        let at_north = self.row == 0;
        let close = at_east || (!at_north && rng.double() < 0.5);

        if close {
            if !at_north {
                let pick = rng.below(self.run.len() as u32) as usize;
                let that = self.run[pick];
                m.carve(self.row, that, DIR_N);
                m.set_head(self.row, that);
            }
            self.run.clear();
        } else {
            m.carve(self.row, self.col, DIR_E);
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
