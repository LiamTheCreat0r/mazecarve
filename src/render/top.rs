//! The top-down view: walls as dots, strokes, ASCII texture or solid blocks.

use crate::canvas::Canvas;
use crate::maze::{DIR_E, DIR_N, DIR_S, DIR_W, Maze, Role, wall_bit};
use crate::render::Render;
use crate::theme::Style;

const BOX_LINE: [char; 16] = [
    '─', // 0000 isolated
    '│', // 0001 N
    '─', // 0010 E
    '└', // 0011 NE
    '│', // 0100 S
    '│', // 0101 NS
    '┌', // 0110 SE
    '├', // 0111 NSE
    '─', // 1000 W
    '┘', // 1001 NW
    '─', // 1010 EW
    '┴', // 1011 NEW
    '┐', // 1100 SW
    '┤', // 1101 NSW
    '┬', // 1110 SEW
    '┼', // 1111 NSEW
];

fn path_char(r: &Render, role: Role) -> char {
    if r.use_color {
        return ' ';
    }
    match role {
        Role::Head => '@',
        Role::Trail1 | Role::Trail2 => ':',
        Role::Unvis => '.',
        _ => ' ',
    }
}

fn cv_wall(cv: &Canvas, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= cv.w || y >= cv.h {
        return false;
    }
    cv.role[cv.at(x, y)] == Role::Wall
}

// Wall texture: seeded ASCII glyphs, grouped by stroke direction so the
// maze keeps its shape while the walls look hand-typed.
const TEX_HORZ: &[u8] = b"-=~_:."; // E-W runs
const TEX_VERT: &[u8] = b"|!:'"; // N-S runs
const TEX_JOINT: &[u8] = b"+#*x@%"; // corners, tees, caps
const TEX_CROSS: &[u8] = b"+#x*"; // NSEW crossing

fn tex_hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h = seed ^ 0x9e37_79b9;
    h ^= x.wrapping_mul(0x85eb_ca6b);
    h = h.rotate_left(13);
    h ^= y.wrapping_mul(0xc2b2_ae35);
    h = h.rotate_left(13);
    h ^= h >> 16;
    h
}

fn tex_char(r: &Render, bits: u32, x: i32, y: i32) -> char {
    let ew = bits & 10 == 10; // walls to the east and west
    let ns = bits & 5 == 5; // walls to the north and south
    let set = match (ew, ns) {
        (true, false) => TEX_HORZ,
        (false, true) => TEX_VERT,
        (true, true) => TEX_CROSS,
        _ => TEX_JOINT,
    };
    let i = tex_hash(x as u32, y as u32, r.tex_seed) as usize % set.len();
    set[i] as char
}

fn wall_char(cv: &Canvas, r: &Render, x: i32, y: i32) -> char {
    if r.style == Style::AsciiDots {
        return '.';
    }
    if r.style == Style::Block {
        return '█';
    }
    // Walls thicker than one character read better as a solid mass.
    if r.wall_width > 1 {
        return match r.style {
            Style::Dots => '⣿',
            Style::Ascii | Style::Texture => '#',
            _ => '█',
        };
    }
    let bits = cv_wall(cv, x, y - 1) as u32
        | (cv_wall(cv, x + 1, y) as u32) << 1
        | (cv_wall(cv, x, y + 1) as u32) << 2
        | (cv_wall(cv, x - 1, y) as u32) << 3;
    match r.style {
        Style::Dots => dots_char(bits),
        Style::Texture => tex_char(r, bits, x, y),
        Style::Ascii => match bits {
            1 | 4 | 5 => '|',
            2 | 8 | 10 => '-',
            _ => '+',
        },
        _ => BOX_LINE[bits as usize],
    }
}

/// A braille cell for a wall, given which neighbours are walls too
/// (1 = N, 2 = E, 4 = S, 8 = W).  Runs and junctions are a full cell;
/// where a wall ends, the corner dots on its open side are dropped so
/// the tip looks carved in rather than cut off square.
///
/// Braille dots are numbered (bit values in hex):
///
/// ```text
///     1 4      01 08
///     2 5      02 10
///     3 6      04 20
///     7 8      40 80
/// ```
fn dots_char(bits: u32) -> char {
    let dots = match bits {
        8 => 0x77, // runs west, ends to the east:  ⡷
        2 => 0xBE, // runs east, ends to the west:  ⢾
        1 => 0x3F, // runs north, ends below:       ⠿
        4 => 0xF6, // runs south, ends above:       ⣶
        0 => 0x36, // a lone pillar, all corners off: ⠶
        _ => 0xFF, // runs and junctions:           ⣿
    };
    char::from_u32(0x2800 + dots).unwrap_or('⣿')
}

pub fn build(cv: &mut Canvas, m: &Maze, r: &Render) {
    let ww = r.wall_width;
    let cc = r.corridor;
    let fill = match r.style {
        Style::Dots => '⣿',
        Style::AsciiDots => '.',
        Style::Ascii | Style::Texture => '#',
        _ => '█',
    };
    cv.ch.fill(fill);
    cv.role.fill(Role::Wall);

    for row in 0..m.rows {
        let y0 = ww + row * (cc + ww);
        if y0 - ww >= cv.h {
            break;
        }
        for col in 0..m.cols {
            let x0 = ww + col * (cc + ww);
            if x0 - ww >= cv.w {
                break;
            }
            let wl = m.walls[m.idx(row, col)];
            let role = m.cell_role(row, col);
            let pc = path_char(r, role);

            cv.paint(x0, y0, cc, cc, pc, role);
            if wl & wall_bit(DIR_N) == 0 {
                cv.paint(x0, y0 - ww, cc, ww, pc, role);
            }
            if wl & wall_bit(DIR_S) == 0 {
                cv.paint(x0, y0 + cc, cc, ww, pc, role);
            }
            if wl & wall_bit(DIR_W) == 0 {
                cv.paint(x0 - ww, y0, ww, cc, pc, role);
            }
            if wl & wall_bit(DIR_E) == 0 {
                cv.paint(x0 + cc, y0, ww, cc, pc, role);
            }
        }
    }

    // Strokes, so corners and junctions meet properly.
    if r.style != Style::Block && r.wall_width == 1 {
        for y in 0..cv.h {
            for x in 0..cv.w {
                let i = cv.at(x, y);
                if cv.role[i] == Role::Wall {
                    cv.ch[i] = wall_char(cv, r, x, y);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::maze::Maze;
    use crate::options::Options;
    use crate::render::print_text;
    use crate::rng::Rng;
    use crate::theme::Style;

    /// The dots style draws every wall, thin or thick, as a full braille cell.
    #[test]
    fn dots_style_draws_only_dots() {
        for ww in [1, 3] {
            let mut o = Options::default();
            o.style = Style::Dots;
            o.wall_width = ww;
            let mut m = Maze::new(6, 9);
            let mut rng = Rng::new(3);
            let mut algo = (crate::algos::find("prim").unwrap().create)(&mut m, &mut rng, "mix");
            while !algo.step(&mut m, &mut rng) {}
            m.set_head(-1, -1);
            m.tick += 100;
            let text = print_text(&m, &o);
            assert!(text.contains('⣿'));
            // Thin walls round off their tips; thick walls stay solid.
            let tips = text.chars().any(|c| matches!(c, '\u{2800}'..='\u{28FE}'));
            assert_eq!(tips, ww == 1, "{text}");
            assert!(
                text.chars()
                    .all(|c| matches!(c, '\u{2800}'..='\u{28FF}' | ' ' | '\n')),
                "{text}"
            );
        }
    }
}
