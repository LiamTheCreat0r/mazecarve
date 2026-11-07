//! Randomized Kruskal's algorithm: every inner wall is shuffled; edges
//! that would close a cycle are dropped.

use super::Algo;
use crate::maze::{DIR_E, DIR_S, Maze};
use crate::rng::Rng;

pub struct Kruskal {
    edges: Vec<(usize, usize)>,
    pos: usize,
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl Kruskal {
    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    let cells = (m.rows * m.cols) as usize;
    m.reset(true);

    let mut edges = Vec::new();
    for r in 0..m.rows {
        for c in 0..m.cols {
            let a = m.idx(r, c);
            if c + 1 < m.cols {
                edges.push((a, a + 1));
            }
            if r + 1 < m.rows {
                edges.push((a, a + m.cols as usize));
            }
        }
    }
    // Fisher-Yates over the edge pairs.
    for i in (1..edges.len()).rev() {
        let j = rng.below(i as u32 + 1) as usize;
        edges.swap(i, j);
    }

    Box::new(Kruskal {
        edges,
        pos: 0,
        parent: (0..cells).collect(),
        rank: vec![0; cells],
    })
}

impl Algo for Kruskal {
    fn step(&mut self, m: &mut Maze, _rng: &mut Rng) -> bool {
        // Scan until an edge is taken, so every step shows growth.
        while self.pos < self.edges.len() {
            let (a, b) = self.edges[self.pos];
            self.pos += 1;
            if self.find(a) != self.find(b) {
                let cols = m.cols as usize;
                let (ar, ac) = ((a / cols) as i32, (a % cols) as i32);
                let (br, bc) = ((b / cols) as i32, (b % cols) as i32);
                let dir = if br == ar { DIR_E } else { DIR_S };
                self.union(a, b);
                m.carve(ar, ac, dir);
                m.mark(ar, ac);
                m.mark(br, bc);
                m.set_head(br, bc);
                return false;
            }
        }
        m.set_head(-1, -1);
        true
    }
}
