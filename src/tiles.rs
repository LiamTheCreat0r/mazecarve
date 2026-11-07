//! The maze expanded into tiles, and the grid rotation the isometric
//! and oblique views share.
//!
//! A maze of rows x cols cells becomes a (2*cols+1) x (2*rows+1) tile
//! grid: odd/odd tiles are cell interiors, even tiles are the wall
//! segments (and corner pillars) between them.

use crate::maze::{DIR_E, DIR_N, DIR_S, DIR_W, Maze, Role};

pub struct Tiles {
    pub w: i32,
    pub h: i32,
    /// true => wall block, false => floor.
    pub wall: Vec<bool>,
    /// Colour role for floor tiles.
    pub role: Vec<Role>,
}

/// Wall segment between (r,c-1) and (r,c).
fn seg_hwall(m: &Maze, r: i32, c: i32) -> bool {
    // Bottom-edge pillars ask about row `rows`, which does not exist.
    // Their other three neighbours always include a border wall, so the
    // answer does not depend on this term.
    if r >= m.rows {
        return false;
    }
    if c > 0 && c < m.cols {
        m.has_wall(r, c - 1, DIR_E) || m.has_wall(r, c, DIR_W)
    } else if c <= 0 {
        m.has_wall(r, 0, DIR_W)
    } else {
        m.has_wall(r, m.cols - 1, DIR_E)
    }
}

/// Wall segment between (r-1,c) and (r,c).
fn seg_vwall(m: &Maze, r: i32, c: i32) -> bool {
    // Right-edge pillars: see seg_hwall.
    if c >= m.cols {
        return false;
    }
    if r > 0 && r < m.rows {
        m.has_wall(r - 1, c, DIR_S) || m.has_wall(r, c, DIR_N)
    } else if r <= 0 {
        m.has_wall(0, c, DIR_N)
    } else {
        m.has_wall(m.rows - 1, c, DIR_S)
    }
}

impl Tiles {
    pub fn build(m: &Maze) -> Self {
        let tw = 2 * m.cols + 1;
        let th = 2 * m.rows + 1;
        let n = (tw * th) as usize;
        let mut t = Tiles {
            w: tw,
            h: th,
            wall: vec![false; n],
            role: vec![Role::Wall; n],
        };
        for y in 0..th {
            for x in 0..tw {
                let i = (y * tw + x) as usize;
                let (oddx, oddy) = (x & 1 == 1, y & 1 == 1);
                if oddx && oddy {
                    // Cell interior.  Not yet carved => still solid rock,
                    // which makes live carving look like digging.
                    let (r, c) = (y / 2, x / 2);
                    if m.vis[m.idx(r, c)] {
                        t.role[i] = m.cell_role(r, c);
                    } else {
                        t.wall[i] = true;
                    }
                } else if !oddx && oddy {
                    t.wall[i] = seg_hwall(m, y / 2, x / 2);
                } else if oddx && !oddy {
                    t.wall[i] = seg_vwall(m, y / 2, x / 2);
                } else {
                    // Corner pillar: solid when any of the four segments
                    // touching it is a wall.
                    t.wall[i] = (y >= 1 && seg_hwall(m, (y - 1) / 2, x / 2))
                        || seg_hwall(m, y / 2, x / 2)
                        || (x >= 1 && seg_vwall(m, y / 2, (x - 1) / 2))
                        || seg_vwall(m, y / 2, x / 2);
                }
            }
        }
        t
    }
}

/// Rotated coords back to original tiles: inverse of rotating (x, y) in a
/// tw x th grid by `rot` 90 degree steps.
pub fn unrotate(rot: i32, tw: i32, th: i32, rx: i32, ry: i32) -> (i32, i32) {
    match rot & 3 {
        1 => (tw - 1 - ry, rx),
        2 => (tw - 1 - rx, th - 1 - ry),
        3 => (ry, th - 1 - rx),
        _ => (rx, ry),
    }
}
