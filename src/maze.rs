//! The cell grid, wall bookkeeping and the post-generation decoration
//! passes (braid, endpoints).

use crate::rng::Rng;

pub const DIR_N: usize = 0;
pub const DIR_E: usize = 1;
pub const DIR_S: usize = 2;
pub const DIR_W: usize = 3;

#[inline]
pub const fn wall_bit(d: usize) -> u8 {
    1 << d
}

#[inline]
pub const fn opposite(d: usize) -> usize {
    (d + 2) & 3
}

/// Length of the fading colour trail left behind the generator head.
pub const TRAIL_1: i32 = 8;
pub const TRAIL_2: i32 = 26;

/// Colour roles, in the order the themes and `--colors` use.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Role {
    #[default]
    Wall = 0,
    Path,
    Head,
    Trail1,
    Trail2,
    Unvis,
    Endpoint,
    /// Isometric wall faces: top, left, right.
    WTop,
    WLeft,
    WRight,
    /// Untouched canvas background.
    Bg,
}

pub const ROLE_COUNT: usize = 11;

pub struct Maze {
    pub rows: i32,
    pub cols: i32,
    /// `wall_bit(dir)` set => wall present.
    pub walls: Vec<u8>,
    /// true once the cell has been carved/visited.
    pub vis: Vec<bool>,
    /// Generation tick of first visit, else 0.
    pub stamp: Vec<i32>,
    /// Active cell, -1 when there is none.
    pub head_r: i32,
    pub head_c: i32,
    pub tick: u32,
    pub ep1_r: i32,
    pub ep1_c: i32,
    pub ep2_r: i32,
    pub ep2_c: i32,
}

impl Maze {
    pub fn new(rows: i32, cols: i32) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        let n = (rows * cols) as usize;
        let mut m = Maze {
            rows,
            cols,
            walls: vec![0; n],
            vis: vec![false; n],
            stamp: vec![0; n],
            head_r: -1,
            head_c: -1,
            tick: 0,
            ep1_r: -1,
            ep1_c: -1,
            ep2_r: -1,
            ep2_c: -1,
        };
        m.reset(true);
        m
    }

    #[inline]
    pub fn contains(&self, r: i32, c: i32) -> bool {
        r >= 0 && r < self.rows && c >= 0 && c < self.cols
    }

    #[inline]
    pub fn idx(&self, r: i32, c: i32) -> usize {
        (r * self.cols + c) as usize
    }

    /// Neighbour of (r,c) in direction d, when it is inside the grid.
    #[inline]
    pub fn nb(&self, r: i32, c: i32, d: usize) -> Option<(i32, i32)> {
        let (nr, nc) = step(r, c, d);
        self.contains(nr, nc).then_some((nr, nc))
    }

    /// closed => every wall present (carving algorithms);
    /// otherwise only the border walls (algorithms that build walls).
    pub fn reset(&mut self, closed: bool) {
        for r in 0..self.rows {
            for c in 0..self.cols {
                let w = if closed {
                    0x0F
                } else {
                    let mut w = 0;
                    if r == 0 {
                        w |= wall_bit(DIR_N);
                    }
                    if r == self.rows - 1 {
                        w |= wall_bit(DIR_S);
                    }
                    if c == 0 {
                        w |= wall_bit(DIR_W);
                    }
                    if c == self.cols - 1 {
                        w |= wall_bit(DIR_E);
                    }
                    w
                };
                let i = self.idx(r, c);
                self.walls[i] = w;
            }
        }
        self.vis.fill(false);
        self.stamp.fill(0);
        self.head_r = -1;
        self.head_c = -1;
        self.ep1_r = -1;
        self.ep1_c = -1;
        self.ep2_r = -1;
        self.ep2_c = -1;
        self.tick = 0;
    }

    #[inline]
    pub fn has_wall(&self, r: i32, c: i32, d: usize) -> bool {
        self.walls[self.idx(r, c)] & wall_bit(d) != 0
    }

    /// Open the passage between (r,c) and its neighbour in `dir`.  Both
    /// sides of the wall are updated so the grid never gets out of sync.
    pub fn carve(&mut self, r: i32, c: i32, dir: usize) {
        let Some((nr, nc)) = self.nb(r, c, dir) else {
            return;
        };
        let a = self.idx(r, c);
        let b = self.idx(nr, nc);
        self.walls[a] &= !wall_bit(dir);
        self.walls[b] &= !wall_bit(opposite(dir));
    }

    /// Put a wall back (used by recursive division).
    pub fn wall(&mut self, r: i32, c: i32, dir: usize) {
        let Some((nr, nc)) = self.nb(r, c, dir) else {
            return;
        };
        let a = self.idx(r, c);
        let b = self.idx(nr, nc);
        self.walls[a] |= wall_bit(dir);
        self.walls[b] |= wall_bit(opposite(dir));
    }

    pub fn mark(&mut self, r: i32, c: i32) {
        if !self.contains(r, c) {
            return;
        }
        let i = self.idx(r, c);
        if !self.vis[i] {
            self.vis[i] = true;
            self.stamp[i] = self.tick as i32;
        }
    }

    #[inline]
    pub fn set_head(&mut self, r: i32, c: i32) {
        self.head_r = r;
        self.head_c = c;
    }

    /// Colour role of a cell, following the generator head and its trail.
    pub fn cell_role(&self, r: i32, c: i32) -> Role {
        let i = self.idx(r, c);
        let mut role = if !self.vis[i] {
            Role::Unvis
        } else if r == self.head_r && c == self.head_c {
            Role::Head
        } else {
            let age = (self.tick as i32).wrapping_sub(self.stamp[i]);
            if (0..TRAIL_1).contains(&age) {
                Role::Trail1
            } else if (0..TRAIL_2).contains(&age) {
                Role::Trail2
            } else {
                Role::Path
            }
        };
        if (r == self.ep1_r && c == self.ep1_c) || (r == self.ep2_r && c == self.ep2_c) {
            role = Role::Endpoint;
        }
        role
    }

    fn open_count(&self, r: i32, c: i32) -> i32 {
        let w = self.walls[self.idx(r, c)];
        (0..4)
            .filter(|&d| w & wall_bit(d) == 0 && self.nb(r, c, d).is_some())
            .count() as i32
    }

    /// Remove `pct` percent of the dead ends so the maze grows loops.
    /// A dead end with no exits at all is stitched into the maze too.
    pub fn braid(&mut self, pct: i32, rng: &mut Rng) {
        if pct <= 0 {
            return;
        }
        let pct = pct.min(100);

        let mut cand: Vec<usize> = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.open_count(r, c) <= 1 {
                    cand.push(self.idx(r, c));
                }
            }
        }
        let n = cand.len();
        let target = (n as i64 * pct as i64 / 100) as usize;
        let mut done = 0;
        let mut i = 0;
        // Partial Fisher-Yates so we only touch the prefix we need.
        while i < n && done < target {
            let j = i + rng.below((n - i) as u32) as usize;
            cand.swap(i, j);

            let idx = cand[i];
            let r = idx as i32 / self.cols;
            let c = idx as i32 % self.cols;

            let mut order = [0usize, 1, 2, 3];
            for d in (1..4).rev() {
                let k = rng.below(d as u32 + 1) as usize;
                order.swap(d, k);
            }
            for &d in &order {
                if self.walls[idx] & wall_bit(d) == 0 {
                    continue;
                }
                if self.nb(r, c, d).is_none() {
                    continue;
                }
                self.carve(r, c, d);
                done += 1;
                break;
            }
            i += 1;
        }
    }

    /// Open one gap in the top border and one in the bottom border.
    pub fn endpoints(&mut self, rng: &mut Rng) {
        let last = self.rows - 1;

        let best = (0..self.cols)
            .find(|&c| self.open_count(0, c) == 1)
            .unwrap_or_else(|| rng.below(self.cols as u32) as i32);
        // Border gaps have no neighbour, so clear the bit directly.
        let i = self.idx(0, best);
        self.walls[i] &= !wall_bit(DIR_N);
        self.ep1_r = 0;
        self.ep1_c = best;

        let best = (0..self.cols)
            .find(|&c| self.open_count(last, c) == 1)
            .unwrap_or_else(|| rng.below(self.cols as u32) as i32);
        let i = self.idx(last, best);
        self.walls[i] &= !wall_bit(DIR_S);
        self.ep2_r = last;
        self.ep2_c = best;
    }
}

/// (r,c) moved one step in direction d, without bounds checks.
#[inline]
pub fn step(r: i32, c: i32, d: usize) -> (i32, i32) {
    match d {
        DIR_N => (r - 1, c),
        DIR_S => (r + 1, c),
        DIR_E => (r, c + 1),
        _ => (r, c - 1),
    }
}
