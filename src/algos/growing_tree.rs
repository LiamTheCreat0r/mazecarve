//! Growing tree algorithm with a selectable strategy (newest / oldest /
//! random / mix).

use super::Algo;
use crate::maze::Maze;
use crate::rng::Rng;

#[derive(Clone, Copy)]
enum Strategy {
    Newest,
    Oldest,
    Random,
    Mix,
}

pub struct GrowingTree {
    list: Vec<usize>,
    /// Flips each step under the "mix" strategy.
    toggle: u32,
    strategy: Strategy,
}

pub fn create(m: &mut Maze, rng: &mut Rng, strategy: &str) -> Box<dyn Algo> {
    let strategy = match strategy {
        "newest" => Strategy::Newest,
        "oldest" => Strategy::Oldest,
        "random" => Strategy::Random,
        _ => Strategy::Mix,
    };
    m.reset(true);
    let r = rng.below(m.rows as u32) as i32;
    let c = rng.below(m.cols as u32) as i32;
    m.mark(r, c);
    m.set_head(r, c);
    Box::new(GrowingTree {
        list: vec![m.idx(r, c)],
        toggle: 0,
        strategy,
    })
}

impl Algo for GrowingTree {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        if self.list.is_empty() {
            m.set_head(-1, -1);
            return true;
        }
        let len = self.list.len();
        let pick = match self.strategy {
            Strategy::Oldest => 0,
            Strategy::Random => rng.below(len as u32) as usize,
            Strategy::Mix => {
                let p = if self.toggle & 1 != 0 {
                    rng.below(len as u32) as usize
                } else {
                    len - 1
                };
                self.toggle = self.toggle.wrapping_add(1);
                p
            }
            Strategy::Newest => len - 1,
        };

        let cols = m.cols;
        let r = self.list[pick] as i32 / cols;
        let c = self.list[pick] as i32 % cols;

        let mut dirs = [0usize; 4];
        let mut n = 0;
        for d in 0..4 {
            if let Some((nr, nc)) = m.nb(r, c, d) {
                if !m.vis[m.idx(nr, nc)] {
                    dirs[n] = d;
                    n += 1;
                }
            }
        }

        if n == 0 {
            // Dead end: drop from the list.
            if pick == len - 1 {
                self.list.pop();
            } else if pick == 0 {
                self.list.remove(0);
            } else {
                self.list.swap_remove(pick);
            }
            let Some(&last) = self.list.last() else {
                m.set_head(-1, -1);
                return true;
            };
            m.set_head(last as i32 / cols, last as i32 % cols);
            return false;
        }

        let d = dirs[rng.below(n as u32) as usize];
        let (nr, nc) = m.nb(r, c, d).expect("checked above");
        m.carve(r, c, d);
        m.mark(nr, nc);
        m.set_head(nr, nc);
        self.list.push(m.idx(nr, nc));
        false
    }
}
