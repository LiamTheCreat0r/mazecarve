//! Aldous-Broder random walk: unbiased but slow, hypnotic to watch.

use super::Algo;
use crate::maze::Maze;
use crate::rng::Rng;

pub struct AldousBroder {
    r: i32,
    c: i32,
    left: i32,
}

pub fn create(m: &mut Maze, rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(true);
    let r = rng.below(m.rows as u32) as i32;
    let c = rng.below(m.cols as u32) as i32;
    m.mark(r, c);
    m.set_head(r, c);
    Box::new(AldousBroder {
        r,
        c,
        left: m.rows * m.cols - 1,
    })
}

impl Algo for AldousBroder {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        if self.left <= 0 {
            m.set_head(-1, -1);
            return true;
        }
        let (or, oc) = (self.r, self.c);
        let d = rng.below(4) as usize;
        let Some((nr, nc)) = m.nb(or, oc, d) else {
            m.set_head(or, oc); // bumped into the border
            return false;
        };
        self.r = nr;
        self.c = nc;
        m.set_head(nr, nc);
        if !m.vis[m.idx(nr, nc)] {
            m.carve(or, oc, d);
            m.mark(nr, nc);
            self.left -= 1;
        }
        false
    }
}
