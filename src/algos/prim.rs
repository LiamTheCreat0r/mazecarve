//! Randomized Prim's algorithm on cells: bushy, full of short dead ends,
//! grows outward like a stain.

use super::Algo;
use crate::maze::Maze;
use crate::rng::Rng;

pub struct Prim {
    frontier: Vec<usize>,
    /// true while queued
    queued: Vec<bool>,
}

impl Prim {
    fn add(&mut self, idx: usize) {
        if self.queued[idx] {
            return;
        }
        self.queued[idx] = true;
        self.frontier.push(idx);
    }
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    let mut s = Prim {
        frontier: Vec::new(),
        queued: vec![false; (m.rows * m.cols) as usize],
    };
    m.reset(true);
    let r = rng.below(m.rows as u32) as i32;
    let c = rng.below(m.cols as u32) as i32;
    m.mark(r, c);
    m.set_head(r, c);
    for d in 0..4 {
        if let Some((nr, nc)) = m.nb(r, c, d) {
            s.add(m.idx(nr, nc));
        }
    }
    Box::new(s)
}

impl Algo for Prim {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        if self.frontier.is_empty() {
            return true;
        }
        let pick = rng.below(self.frontier.len() as u32) as usize;
        let idx = self.frontier[pick];
        let r = idx as i32 / m.cols;
        let c = idx as i32 % m.cols;

        // At least one neighbour is carved already by construction.
        let mut dirs = [0usize; 4];
        let mut n = 0;
        for d in 0..4 {
            if let Some((nr, nc)) = m.nb(r, c, d)
                && m.vis[m.idx(nr, nc)]
            {
                dirs[n] = d;
                n += 1;
            }
        }

        if n > 0 {
            let sd = dirs[rng.below(n as u32) as usize];
            m.carve(r, c, sd);
            m.mark(r, c);
            m.set_head(r, c);
            for d in 0..4 {
                if let Some((nr, nc)) = m.nb(r, c, d) {
                    let ni = m.idx(nr, nc);
                    if !m.vis[ni] {
                        self.add(ni);
                    }
                }
            }
        }

        // Consumed: swap-remove.
        self.frontier.swap_remove(pick);
        self.queued[idx] = false;
        false
    }
}
