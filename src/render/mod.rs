//! Turning a grid of walls into characters and colours.  The same canvas
//! builders drive the live view and the plain `mazecarve -p` dump.

mod iso;
mod top;

use crate::canvas::Canvas;
use crate::maze::ROLE_COUNT;
use crate::options::Options;
use crate::theme::{self, Style, View};

pub use iso::{iso_size, oblique_size};

/// Everything the builders need to know about how to draw.
pub struct Render {
    pub style: Style,
    pub wall_width: i32,
    pub corridor: i32,
    pub use_color: bool,
    /// Colours the terminal can show (8, 16 or 256).
    pub colors: u32,
    /// Per-maze seed for the wall texture.
    pub tex_seed: u32,
    /// Resolved xterm colour for every role.
    pub col: [u8; ROLE_COUNT],
}

impl Render {
    /// Rendering without colour, for printed output.
    pub fn plain(o: &Options) -> Self {
        Render {
            style: o.style,
            wall_width: o.wall_width,
            corridor: o.corridor,
            use_color: false,
            colors: 0,
            tex_seed: o.seed as u32,
            col: [0; ROLE_COUNT],
        }
    }

    pub fn for_terminal(o: &Options, use_color: bool, colors: u32) -> Self {
        let pal = theme::palette(&o.theme, o.colors.as_deref());
        let mut col = [0u8; ROLE_COUNT];
        for (c, &p) in col.iter_mut().zip(pal.iter()) {
            *c = theme::nearest(p, colors);
        }
        Render {
            use_color,
            colors,
            col,
            ..Render::plain(o)
        }
    }

    /// Solid colour slabs (spaces with fg == bg) only read well when the
    /// terminal can tell the three face shades apart.
    pub fn solid_ok(&self) -> bool {
        self.use_color && self.colors >= 16
    }
}

/// Size in characters of the top-down drawing of a rows x cols maze.
pub fn top_size(o: &Options, rows: i32, cols: i32) -> (i32, i32) {
    (
        cols * o.corridor + (cols + 1) * o.wall_width,
        rows * o.corridor + (rows + 1) * o.wall_width,
    )
}

/// Build the canvas for `view`.
pub fn build(view: View, w: i32, h: i32, m: &crate::maze::Maze, o: &Options, r: &Render) -> Canvas {
    let mut cv = Canvas::new(w, h);
    match view {
        View::Iso => iso::iso_build(&mut cv, m, o, r),
        View::Oblique => iso::oblique_build(&mut cv, m, o, r),
        View::Top => top::build(&mut cv, m, r),
    }
    cv
}

// ------------------------------------------------------------------
// Message box
// ------------------------------------------------------------------

const MAX_MSG_LINES: usize = 12;
/// Longest wrapped line inside the box.
const MSG_MAX_W: i32 = 40;
/// The original kept the message in a 512 wide-char buffer.
const MSG_MAX_CHARS: usize = 511;

pub struct Msg {
    pub text: Vec<char>,
    /// (start, len) of every wrapped line.
    pub lines: Vec<(usize, usize)>,
    pub w: i32,
    pub h: i32,
}

fn wrap_text(s: &[char], width: i32) -> Vec<(usize, usize)> {
    let width = width.max(1) as usize;
    let len = s.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < len && out.len() < MAX_MSG_LINES {
        if len - i <= width {
            out.push((i, len - i));
            break;
        }
        let cut = i + width;
        let mut k = cut;
        while k > i && s[k] != ' ' {
            k -= 1;
        }
        if k == i {
            // No space: hard break.
            out.push((i, width));
            i += width;
        } else {
            out.push((i, k - i));
            i = k + 1;
            while i < len && s[i] == ' ' {
                i += 1;
            }
        }
    }
    if out.is_empty() {
        out.push((0, 0));
    }
    out
}

/// The message wrapped to fit `avail` columns, or None when there is
/// nothing to draw (no message, or no room at all).
pub fn msg_prepare(o: &Options, avail: i32) -> Option<Msg> {
    let text: Vec<char> = o.message.as_deref()?.chars().take(MSG_MAX_CHARS).collect();
    if text.is_empty() {
        return None;
    }
    // One-column margin each side.
    let width = (avail - 4).min(MSG_MAX_W);
    if width < 1 {
        return None;
    }
    let lines = wrap_text(&text, width);
    let h = lines.len() as i32 + 2;
    let mut w = width + 2;
    for &(_, l) in &lines {
        w = w.max(l as i32 + 2);
    }
    let w = w.min(avail);
    if w < 3 {
        return None;
    }
    Some(Msg { text, lines, w, h })
}

/// Geometry (y, x, w, h) of the message box on a tr x tc screen: its own
/// band at the bottom.
pub fn message_box_geom(o: &Options, tr: i32, tc: i32) -> Option<(i32, i32, i32, i32)> {
    let mb = msg_prepare(o, tc)?;
    let y = (tr - mb.h).max(0);
    let x = ((tc - mb.w) / 2).max(0);
    Some((y, x, mb.w, mb.h))
}

/// Box drawing characters: (horiz, vert, tl, tr, bl, br).
pub fn msg_chars(style: Style) -> (char, char, char, char, char, char) {
    if matches!(style, Style::Ascii | Style::Texture) {
        ('-', '|', '+', '+', '+', '+')
    } else {
        ('─', '│', '┌', '┐', '└', '┘')
    }
}

/// The box as rows of characters, `mb.w` wide.
pub fn msg_rows(mb: &Msg, style: Style) -> Vec<Vec<char>> {
    let (horiz, vert, tl, tr, bl, br) = msg_chars(style);
    (0..mb.h)
        .map(|j| {
            (0..mb.w)
                .map(|i| {
                    let last = i == mb.w - 1;
                    if j == 0 {
                        if i == 0 {
                            tl
                        } else if last {
                            tr
                        } else {
                            horiz
                        }
                    } else if j == mb.h - 1 {
                        if i == 0 {
                            bl
                        } else if last {
                            br
                        } else {
                            horiz
                        }
                    } else if i == 0 || last {
                        vert
                    } else {
                        let (start, len) = mb.lines[(j - 1) as usize];
                        if ((i - 1) as usize) < len {
                            mb.text[start + (i - 1) as usize]
                        } else {
                            ' '
                        }
                    }
                })
                .collect()
        })
        .collect()
}

/// Printed output: the maze, then the same message box the live view
/// shows, below it.
pub fn print_text(m: &crate::maze::Maze, o: &Options) -> String {
    let r = Render::plain(o);
    let (w, h) = match o.view_mode {
        View::Iso => iso_size(o, m.rows, m.cols),
        View::Oblique => oblique_size(o, m.rows, m.cols),
        View::Top => top_size(o, m.rows, m.cols),
    };
    let cv = build(o.view_mode, w, h, m, o, &r);
    let mut out = String::new();
    cv.write_text(&mut out);
    if let Some(mb) = msg_prepare(o, cv.w) {
        out.push('\n');
        for row in msg_rows(&mb, o.style) {
            out.extend(row);
            out.push('\n');
        }
    }
    out
}
