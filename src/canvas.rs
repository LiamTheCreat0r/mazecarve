//! The off-screen buffer every view draws into.  A frame is built here
//! and then flushed to the terminal (or stdout) in one pass, which keeps
//! the isometric painter's algorithm simple and flicker free.

use crate::maze::Role;

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub ch: Vec<char>,
    pub role: Vec<Role>,
}

impl Canvas {
    /// A blank canvas: the isometric views do not cover every cell of
    /// their bounding box.
    pub fn new(w: i32, h: i32) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        let n = (w * h) as usize;
        Canvas {
            w,
            h,
            ch: vec![' '; n],
            role: vec![Role::Bg; n],
        }
    }

    #[inline]
    pub fn at(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    pub fn put(&mut self, x: i32, y: i32, ch: char, role: Role) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = self.at(x, y);
        self.ch[i] = ch;
        self.role[i] = role;
    }

    pub fn paint(&mut self, mut x: i32, mut y: i32, mut w: i32, mut h: i32, ch: char, role: Role) {
        if x < 0 {
            w += x;
            x = 0;
        }
        if y < 0 {
            h += y;
            y = 0;
        }
        w = w.min(self.w - x);
        h = h.min(self.h - y);
        if w <= 0 || h <= 0 {
            return;
        }
        for j in 0..h {
            let base = self.at(x, y + j);
            self.ch[base..base + w as usize].fill(ch);
            self.role[base..base + w as usize].fill(role);
        }
    }

    /// The canvas as plain text, one line per row.
    pub fn write_text(&self, out: &mut String) {
        for row in self.ch.chunks(self.w as usize) {
            out.extend(row.iter());
            out.push('\n');
        }
    }
}
