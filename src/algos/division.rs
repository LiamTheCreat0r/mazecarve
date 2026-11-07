//! Recursive division: starts with an open field and *adds* walls, so
//! long straight walls appear and the space gets chopped into rooms.

use super::Algo;
use crate::maze::{DIR_E, DIR_S, Maze, TRAIL_2};
use crate::rng::Rng;

pub struct Division {
    /// (r0, r1, c0, c1) per queued rectangle.
    stack: Vec<(i32, i32, i32, i32)>,
}

pub fn create(m: &mut Maze, _rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    m.reset(false); // open field, border walls only
    // Everything is already "carved": light the grid up at once and park
    // the stamps in the past so no colour trail is painted.
    m.vis.fill(true);
    m.stamp.fill(-TRAIL_2 - 1);
    Box::new(Division {
        stack: vec![(0, m.rows - 1, 0, m.cols - 1)],
    })
}

impl Algo for Division {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        while let Some((r0, r1, c0, c1)) = self.stack.pop() {
            let h = r1 - r0 + 1;
            let w = c1 - c0 + 1;
            if h < 1 || w < 1 || (h < 2 && w < 2) {
                continue;
            }
            // Cut along the longer axis; coin-flip when square.
            let horiz = if h > w {
                true
            } else if w > h {
                false
            } else {
                rng.below(2) == 1
            };

            let (hr, hc) = if horiz {
                let k = r0 + rng.below((h - 1) as u32) as i32;
                let gap = c0 + rng.below(w as u32) as i32;
                for i in c0..=c1 {
                    if i != gap {
                        m.wall(k, i, DIR_S);
                    }
                }
                self.stack.push((k + 1, r1, c0, c1));
                self.stack.push((r0, k, c0, c1));
                (k, gap)
            } else {
                let k = c0 + rng.below((w - 1) as u32) as i32;
                let gap = r0 + rng.below(h as u32) as i32;
                for i in r0..=r1 {
                    if i != gap {
                        m.wall(i, k, DIR_E);
                    }
                }
                self.stack.push((r0, r1, k + 1, c1));
                self.stack.push((r0, r1, c0, k));
                (gap, k)
            };
            m.set_head(hr, hc);
            let i = m.idx(hr, hc);
            m.stamp[i] = m.tick as i32;
            return false;
        }
        m.set_head(-1, -1);
        true
    }
}
