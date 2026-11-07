//! The isometric (2:1 dimetric) and oblique views.
//!
//! Both views expand the maze into a tile grid, rotate it (--rotate) and
//! project it with a parallel projection, drawing tiles back to front
//! (painter's algorithm) so walls in front hide what is behind them.
//!
//! 2:1 isometric projection, for a tile at rotated coordinates (rx, ry)
//! and height z (in rows):
//!
//! ```text
//!     screen_col = (rx - ry) * 2
//!     screen_row = (rx + ry) - z
//! ```
//!
//! A tile footprint is a diamond 4 columns wide and 2 rows tall; a wall
//! block is that diamond lifted `wall_height` rows, plus the left and
//! right side faces filling the gap down to the floor.

use crate::canvas::Canvas;
use crate::maze::{Maze, Role};
use crate::options::Options;
use crate::render::Render;
use crate::theme::IsoStyle;
use crate::tiles::{Tiles, unrotate};

/// Glyphs for one iso wall block.  With colour on, faces are spaces in a
/// colour whose foreground and background match, so they read as slabs.
struct IsoGlyphs {
    /// Top face diamond: row a, cols c-1, c.
    top_l: char,
    top_m: char,
    /// Row a+1: col c-2, cols c-1..c, col c+1.
    top_ll: char,
    top_mm: char,
    top_r: char,
    /// Left face: cols c-2, c-1.
    side_l_edge: char,
    side_l_fill: char,
    /// Right face: cols c, c+1.
    side_r_fill: char,
    side_r_edge: char,
}

const ISO_ASCII: IsoGlyphs = IsoGlyphs {
    top_l: '_',
    top_m: '_',
    top_ll: '/',
    top_mm: '_',
    top_r: '\\',
    side_l_edge: '|',
    side_l_fill: ':',
    side_r_fill: '.',
    side_r_edge: '|',
};

const ISO_BOX: IsoGlyphs = IsoGlyphs {
    top_l: '─',
    top_m: '─',
    top_ll: '╱',
    top_mm: '─',
    top_r: '╲',
    side_l_edge: '│',
    side_l_fill: ' ',
    side_r_fill: ' ',
    side_r_edge: '│',
};

const ISO_BLOCK: IsoGlyphs = IsoGlyphs {
    top_l: '█',
    top_m: '█',
    top_ll: '█',
    top_mm: '█',
    top_r: '█',
    side_l_edge: '▓',
    side_l_fill: '▓',
    side_r_fill: '▒',
    side_r_edge: '▒',
};

const ISO_FLAT: IsoGlyphs = IsoGlyphs {
    top_l: ' ',
    top_m: ' ',
    top_ll: ' ',
    top_mm: ' ',
    top_r: ' ',
    side_l_edge: ' ',
    side_l_fill: ' ',
    side_r_fill: ' ',
    side_r_edge: ' ',
};

fn iso_glyphs(style: IsoStyle, solid: bool) -> &'static IsoGlyphs {
    if solid {
        return &ISO_FLAT;
    }
    match style {
        IsoStyle::Box => &ISO_BOX,
        IsoStyle::Block => &ISO_BLOCK,
        _ => &ISO_ASCII,
    }
}

/// Glyph for a floor tile when colour cannot carry the role.
fn iso_floor_char(role: Role) -> char {
    match role {
        Role::Head => '@',
        Role::Trail1 | Role::Trail2 => ':',
        Role::Unvis => '.',
        Role::Endpoint => '*',
        _ => ' ',
    }
}

/// Wall face roles: (top, left/front, right/side).
fn face_roles(o: &Options) -> (Role, Role, Role) {
    if o.shade {
        (Role::WTop, Role::WLeft, Role::WRight)
    } else {
        (Role::Wall, Role::Wall, Role::Wall)
    }
}

/// Draw one tile: floor diamond at z=0, or the wall block standing on it.
#[allow(clippy::too_many_arguments)]
fn iso_tile(
    cv: &mut Canvas,
    o: &Options,
    r: &Render,
    g: &IsoGlyphs,
    c: i32,
    a: i32,
    is_wall: bool,
    role: Role,
) {
    let hh = o.wall_height.max(1);
    let (r_top, r_left, r_right) = face_roles(o);

    if !is_wall && o.floor {
        let diamond = |cv: &mut Canvas, top: char, ll: char, mid: char, rr: char| {
            cv.put(c - 1, a, top, role);
            cv.put(c, a, top, role);
            cv.put(c - 2, a + 1, ll, role);
            cv.put(c - 1, a + 1, mid, role);
            cv.put(c, a + 1, mid, role);
            cv.put(c + 1, a + 1, rr, role);
        };
        if r.solid_ok() {
            diamond(cv, ' ', ' ', ' ', ' ');
        } else if o.iso_style == IsoStyle::Block {
            // Light shade floor so corridors read between the blocks.
            let f = if matches!(role, Role::Path | Role::Endpoint) {
                '░'
            } else {
                iso_floor_char(role)
            };
            diamond(cv, f, f, f, f);
        } else {
            // A faint diamond outline, so corridors read as holes between
            // the solid wall tops, with the role glyph inside it.
            diamond(cv, ' ', g.top_ll, iso_floor_char(role), g.top_r);
        }
        return;
    }
    if !is_wall {
        return;
    }

    // Side faces, bottom row first so the top face overlaps cleanly.
    for k in 0..hh {
        let row = a + 1 - k;
        cv.put(c - 2, row, g.side_l_edge, r_left);
        cv.put(c - 1, row, g.side_l_fill, r_left);
        cv.put(c, row, g.side_r_fill, r_right);
        cv.put(c + 1, row, g.side_r_edge, r_right);
    }
    // Top face, raised by hh rows.
    cv.put(c - 1, a - hh, g.top_l, r_top);
    cv.put(c, a - hh, g.top_m, r_top);
    cv.put(c - 2, a - hh + 1, g.top_ll, r_top);
    cv.put(c - 1, a - hh + 1, g.top_mm, r_top);
    cv.put(c, a - hh + 1, g.top_mm, r_top);
    cv.put(c + 1, a - hh + 1, g.top_r, r_top);
}

pub fn iso_size(o: &Options, rows: i32, cols: i32) -> (i32, i32) {
    let (tw, th) = (2 * cols + 1, 2 * rows + 1);
    let hh = o.wall_height.max(1);
    // Diamond footprint with a 2:1 slope; floor rows + raised tops.
    (2 * (tw + th), tw + th + hh)
}

/// Rotated grid size.
fn rotated(o: &Options, tw: i32, th: i32) -> (i32, i32) {
    if o.rotate & 1 == 1 {
        (th, tw)
    } else {
        (tw, th)
    }
}

pub fn iso_build(cv: &mut Canvas, m: &Maze, o: &Options, r: &Render) {
    let t = Tiles::build(m);
    let (tw, th) = (t.w, t.h);
    let hh = o.wall_height.max(1);
    let (rw, rh) = rotated(o, tw, th);
    // -col_min / -row_min of the projection, so drawing starts inside.
    let col_off = 2 * rh;
    let row_off = hh;
    let g = iso_glyphs(o.iso_style, r.solid_ok());

    // Painter's algorithm: back to front, by increasing rx + ry.
    for s in 0..=(rw + rh - 2) {
        for rx in 0..rw {
            let ry = s - rx;
            if ry < 0 || ry >= rh {
                continue;
            }
            let (x, y) = unrotate(o.rotate, tw, th, rx, ry);
            let i = (y * tw + x) as usize;
            let c = (rx - ry) * 2 + col_off;
            let a = rx + ry + row_off;
            iso_tile(cv, o, r, g, c, a, t.wall[i], t.role[i]);
        }
    }
}

// ------------------------------------------------------------------
// Oblique: front-facing tiles, depth shown by a diagonal offset.  The
// floor stays rectangular; a wall shows a front, a top and a side face.
// ------------------------------------------------------------------

struct OblGlyphs {
    /// Top face, row r-hh.
    top_l: char,
    top_r: char,
    /// The diagonal joining edge.
    side_top: char,
    /// Front face edge and fill.
    front_l: char,
    front_fill: char,
    /// Right face.
    side: char,
}

const OBL_ASCII: OblGlyphs = OblGlyphs {
    top_l: '_',
    top_r: '_',
    side_top: '/',
    front_l: '|',
    front_fill: ':',
    side: '.',
};

const OBL_BOX: OblGlyphs = OblGlyphs {
    top_l: '─',
    top_r: '─',
    side_top: '╱',
    front_l: '│',
    front_fill: ' ',
    side: ' ',
};

const OBL_BLOCK: OblGlyphs = OblGlyphs {
    top_l: '█',
    top_r: '█',
    side_top: '█',
    front_l: '▓',
    front_fill: '▓',
    side: '▒',
};

const OBL_FLAT: OblGlyphs = OblGlyphs {
    top_l: ' ',
    top_r: ' ',
    side_top: ' ',
    front_l: ' ',
    front_fill: ' ',
    side: ' ',
};

fn obl_glyphs(style: IsoStyle, solid: bool) -> &'static OblGlyphs {
    if solid {
        return &OBL_FLAT;
    }
    match style {
        IsoStyle::Box => &OBL_BOX,
        IsoStyle::Block => &OBL_BLOCK,
        _ => &OBL_ASCII,
    }
}

#[allow(clippy::too_many_arguments)]
fn oblique_tile(
    cv: &mut Canvas,
    o: &Options,
    r: &Render,
    g: &OblGlyphs,
    b: i32,
    row: i32,
    is_wall: bool,
    role: Role,
) {
    let hh = o.wall_height.max(1);
    let (r_top, r_front, r_side) = face_roles(o);

    if !is_wall && o.floor {
        let f = if r.solid_ok() {
            ' '
        } else {
            // Visible corridor glyphs; the sheared grid has no room for
            // an outline the way the iso diamonds do.
            match role {
                Role::Head => '@',
                Role::Trail1 | Role::Trail2 => ':',
                Role::Unvis => ',',
                Role::Endpoint => '*',
                _ if o.iso_style == IsoStyle::Block => '░',
                _ => '.',
            }
        };
        cv.put(b, row, f, role);
        cv.put(b + 1, row, f, role);
        return;
    }
    if !is_wall {
        return;
    }

    for k in 0..hh {
        let y = row - k;
        cv.put(b, y, g.front_l, r_front);
        cv.put(b + 1, y, g.front_fill, r_front);
        cv.put(b + 2, y, g.side, r_side);
    }
    // Top face: raised by hh, offset one column right for depth, and
    // joined to the front face with a diagonal.
    cv.put(b, row - hh, g.side_top, r_top);
    cv.put(b + 1, row - hh, g.top_l, r_top);
    cv.put(b + 2, row - hh, g.top_r, r_top);
}

pub fn oblique_size(o: &Options, rows: i32, cols: i32) -> (i32, i32) {
    let (rw, rh) = rotated(o, 2 * cols + 1, 2 * rows + 1);
    let hh = o.wall_height.max(1);
    // 2 cols per tile + depth skew.
    (2 * rw + rh + 2, rh + hh)
}

pub fn oblique_build(cv: &mut Canvas, m: &Maze, o: &Options, r: &Render) {
    let t = Tiles::build(m);
    let (tw, th) = (t.w, t.h);
    let row_off = o.wall_height.max(1);
    let col_off = 1;
    let (rw, rh) = rotated(o, tw, th);
    let g = obl_glyphs(o.iso_style, r.solid_ok());

    // Back to front: rows first (depth), then columns.
    for ry in 0..rh {
        for rx in 0..rw {
            let (x, y) = unrotate(o.rotate, tw, th, rx, ry);
            let i = (y * tw + x) as usize;
            let b = 2 * rx + ry + col_off;
            let row = ry + row_off;
            oblique_tile(cv, o, r, g, b, row, t.wall[i], t.role[i]);
        }
    }
}
