//! Wilson's algorithm: loop-erased random walks, unbiased sampling.

use super::Algo;
use crate::maze::{DIR_E, DIR_N, DIR_S, DIR_W, Maze};
use crate::rng::Rng;

pub struct Wilson {
    /// Cells of the current walk, in order.
    walk: Vec<usize>,
    /// Index inside `walk`, or -1.
    pos: Vec<i32>,
    /// Cells still unvisited.
    left: i32,
    walking: bool,
}

impl Wilson {
    fn clear(&mut self) {
        for &i in &self.walk {
            self.pos[i] = -1;
        }
        self.walk.clear();
    }

    fn push(&mut self, idx: usize) {
        self.pos[idx] = self.walk.len() as i32;
        self.walk.push(idx);
    }
}

fn pick_unvisited(m: &Maze, rng: &mut Rng) -> Option<usize> {
    let total = (m.rows * m.cols) as u32;
    for _ in 0..64 {
        let i = rng.below(total) as usize;
        if !m.vis[i] {
            return Some(i);
        }
    }
    m.vis.iter().position(|&v| !v)
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    let cells = (m.rows * m.cols) as usize;
    m.reset(true);
    let start = rng.below(m.rows as u32 * m.cols as u32) as i32;
    m.mark(start / m.cols, start % m.cols);
    m.set_head(start / m.cols, start % m.cols);
    Box::new(Wilson {
        walk: Vec::new(),
        pos: vec![-1; cells],
        left: m.rows * m.cols - 1,
        walking: false,
    })
}

impl Algo for Wilson {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        if self.left <= 0 {
            m.set_head(-1, -1);
            return true;
        }
        let cols = m.cols as usize;

        if !self.walking {
            let Some(idx) = pick_unvisited(m, rng) else {
                self.left = 0;
                return true;
            };
            self.clear();
            self.push(idx);
            self.walking = true;
            m.set_head((idx / cols) as i32, (idx % cols) as i32);
            return false;
        }

        let cur = *self.walk.last().expect("walk is never empty while walking");
        let d = rng.below(4) as usize;
        let Some((nr, nc)) = m.nb((cur / cols) as i32, (cur % cols) as i32, d) else {
            return false; // bumped into the border
        };
        let n = m.idx(nr, nc);

        if self.pos[n] >= 0 {
            // The walk revisited itself: erase the loop it just made.
            let from = self.pos[n] as usize + 1;
            for &i in &self.walk[from..] {
                self.pos[i] = -1;
            }
            self.walk.truncate(from);
            m.set_head(nr, nc);
            return false;
        }

        if m.vis[n] {
            // Reached the maze: commit the walk.
            self.push(n);
            for w in self.walk.windows(2) {
                let (a, b) = (w[0], w[1]);
                // Compare row/column rather than raw indices, so a single
                // column maze (where a+1 is also the cell below) works.
                let (ar, ac) = (a / cols, a % cols);
                let (br, bc) = (b / cols, b % cols);
                let dir = if br == ar {
                    if bc > ac { DIR_E } else { DIR_W }
                } else if br > ar {
                    DIR_S
                } else {
                    DIR_N
                };
                m.carve((a / cols) as i32, (a % cols) as i32, dir);
            }
            for &idx in &self.walk {
                if !m.vis[idx] {
                    m.vis[idx] = true;
                    m.stamp[idx] = m.tick as i32;
                    self.left -= 1;
                }
            }
            self.walking = false;
            self.clear();
            m.set_head(nr, nc);
            return false;
        }

        self.push(n);
        m.set_head(nr, nc);
        false
    }
}
