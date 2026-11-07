//! Sizing the maze to the terminal and placing it on screen.

use crate::options::Options;
use crate::render::{iso_size, message_box_geom, oblique_size, top_size};
use crate::theme::View;

#[derive(Clone, Copy, Debug, Default)]
pub struct Layout {
    /// Maze dimensions in cells.
    pub rows: i32,
    pub cols: i32,
    /// Rendered size in characters.
    pub cw: i32,
    pub ch: i32,
    /// Top-left corner of the maze on screen.
    pub oy: i32,
    pub ox: i32,
    /// Terminal size used for centring.
    pub tr: i32,
    pub tc: i32,
    /// View actually drawn (iso falls back to top when nothing fits).
    pub view: View,
    /// Message box, zero sized when there is none.
    pub msg_y: i32,
    pub msg_x: i32,
    pub msg_w: i32,
    pub msg_h: i32,
}

/// Projected size of the maze, when the view is not the top-down one.
fn proj_dims(o: &Options, rows: i32, cols: i32) -> Option<(i32, i32)> {
    match o.view_mode {
        View::Iso => Some(iso_size(o, rows, cols)),
        View::Oblique => Some(oblique_size(o, rows, cols)),
        View::Top => None,
    }
}

/// Derive cell dimensions from a terminal size (or honour --size).
pub fn autosize(o: &Options, mut tr: i32, mut tc: i32) -> (i32, i32) {
    let ww = o.wall_width;
    let cc = o.corridor;

    // Leave room for the message box so the whole output fits.
    if let Some((_, _, _, h)) = message_box_geom(o, tr, tc) {
        tr = (tr - h).max(1);
    }

    // Only claim a share of what is left, so the maze floats on a margin.
    // An explicit --size always wins.
    let fill = o.fill.clamp(1, 100);
    tc = (tc * fill / 100).max(1);
    tr = (tr * fill / 100).max(1);

    if o.view_mode != View::Top && o.size_w <= 0 && o.size_h <= 0 {
        // Projected views: the bounding box grows with the maze, so find
        // the largest square maze whose projection still fits.
        let mut n = 1;
        while n < 512 {
            let (w, h) = proj_dims(o, n + 1, n + 1).expect("projected view");
            if w > tc || h > tr {
                break;
            }
            n += 1;
        }
        return (n, n);
    }

    let mut c = if o.size_w > 0 {
        o.size_w
    } else {
        (tc - ww) / (cc + ww)
    };
    let mut r = if o.size_h > 0 {
        o.size_h
    } else {
        (tr - ww) / (cc + ww)
    };

    if o.view_mode != View::Top {
        // One dimension given: keep the maze square.
        if o.size_w > 0 && o.size_h <= 0 {
            r = c;
        } else if o.size_h > 0 && o.size_w <= 0 {
            c = r;
        }
    }
    (r.max(1), c.max(1))
}

/// Place a rows x cols maze on a tr x tc screen.  The origin is clamped so
/// an oversized maze is simply clipped.  A message box gets its own band
/// at the bottom and the maze is centred in what is left.
pub fn place(rows: i32, cols: i32, o: &Options, tr: i32, tc: i32) -> Layout {
    let mut l = Layout {
        rows,
        cols,
        tr,
        tc,
        view: o.view_mode,
        ..Layout::default()
    };

    let mut band = 0;
    if let Some((y, x, w, h)) = message_box_geom(o, tr, tc) {
        (l.msg_y, l.msg_x, l.msg_w, l.msg_h) = (y, x, w, h);
        band = h;
        if band > tr {
            // Box taller than the screen.
            band = tr.max(0);
            l.msg_y = 0;
            l.msg_h = band;
        }
    }

    match proj_dims(o, rows, cols) {
        None => (l.cw, l.ch) = top_size(o, rows, cols),
        Some((w, h)) => {
            (l.cw, l.ch) = (w, h);
            if (w > tc || h > tr - band) && rows <= 1 && cols <= 1 {
                // Not even a minimal projected maze fits: fall back to
                // the top view, which is always small enough to try.
                l.view = View::Top;
                (l.cw, l.ch) = top_size(o, rows, cols);
            }
        }
    }

    l.ox = ((tc - l.cw) / 2).max(0);
    l.oy = (((tr - band) - l.ch) / 2).max(0);
    l
}
