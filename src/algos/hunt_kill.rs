//! Hunt-and-kill: carve at random until boxed in, then scan (visibly,
//! one cell per step) for an unvisited cell touching the maze.

use super::Algo;
use crate::maze::Maze;
use crate::rng::Rng;

pub struct HuntKill {
    r: i32,
    c: i32,
    hunting: bool,
    /// Persistent scan cursor, wraps row-major.
    hr: i32,
    hc: i32,
    /// Cells scanned in the current hunt.
    examined: i32,
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(true);
    let r = rng.below(m.rows as u32) as i32;
    let c = rng.below(m.cols as u32) as i32;
    m.mark(r, c);
    m.set_head(r, c);
    Box::new(HuntKill {
        r,
        c,
        hunting: false,
        hr: 0,
        hc: 0,
        examined: 0,
    })
}

fn dirs_where(m: &Maze, r: i32, c: i32, visited: bool) -> ([usize; 4], usize) {
    let mut dirs = [0usize; 4];
    let mut n = 0;
    for d in 0..4 {
        if let Some((nr, nc)) = m.nb(r, c, d) {
            if m.vis[m.idx(nr, nc)] == visited {
                dirs[n] = d;
                n += 1;
            }
        }
    }
    (dirs, n)
}

impl Algo for HuntKill {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        if !self.hunting {
            // Kill phase: carve at random.
            let (dirs, n) = dirs_where(m, self.r, self.c, false);
            if n > 0 {
                let d = dirs[rng.below(n as u32) as usize];
                let (nr, nc) = m.nb(self.r, self.c, d).expect("checked above");
                m.carve(self.r, self.c, d);
                m.mark(nr, nc);
                self.r = nr;
                self.c = nc;
                m.set_head(nr, nc);
                return false;
            }
            self.hunting = true; // boxed in: fall through to hunt
        }

        // Hunt phase: examine exactly one cell this step.
        let (r, c) = (self.hr, self.hc);
        let idx = m.idx(r, c);
        m.set_head(r, c);
        self.examined += 1;

        self.hc += 1;
        if self.hc >= m.cols {
            self.hc = 0;
            self.hr += 1;
            if self.hr >= m.rows {
                self.hr = 0;
            }
        }

        if !m.vis[idx] {
            let (vn, nv) = dirs_where(m, r, c, true);
            if nv > 0 {
                let d = vn[rng.below(nv as u32) as usize];
                m.carve(r, c, d);
                m.mark(r, c);
                self.r = r;
                self.c = c;
                m.set_head(r, c);
                self.hunting = false;
                self.examined = 0;
                return false;
            }
        }

        // A full pass without a candidate: every cell is accounted for.
        if self.examined >= m.rows * m.cols {
            m.set_head(-1, -1);
            return true;
        }
        false
    }
}
