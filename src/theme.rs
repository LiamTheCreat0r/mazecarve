//! Colour names, palettes and the colour model.
//!
//! Colours are addressed as xterm-256 indices.  Anything the terminal
//! cannot display is remapped to the nearest colour it does have, so the
//! same theme works on an 8-colour terminal as well as a 256-colour one.

use crate::maze::{ROLE_COUNT, Role};

fn xterm_rgb(c: i32) -> (i32, i32, i32) {
    const BASE: [(i32, i32, i32); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    const RAMP: [i32; 6] = [0, 95, 135, 175, 215, 255];
    let c = c.clamp(0, 255);
    if c < 16 {
        BASE[c as usize]
    } else if c < 232 {
        let i = c - 16;
        (
            RAMP[(i / 36) as usize],
            RAMP[((i / 6) % 6) as usize],
            RAMP[(i % 6) as usize],
        )
    } else {
        let v = 8 + (c - 232) * 10;
        (v, v, v)
    }
}

fn rgb_dist2(c: i32, r: i32, g: i32, b: i32) -> i32 {
    let (cr, cg, cb) = xterm_rgb(c);
    let (dr, dg, db) = (cr - r, cg - g, cb - b);
    dr * dr + dg * dg + db * db
}

/// Nearest colour among the first `limit` xterm colours.
fn nearest_in(r: i32, g: i32, b: i32, limit: i32) -> i32 {
    let mut best = 0;
    let mut bestd = i32::MAX;
    for i in 0..limit {
        let d = rgb_dist2(i, r, g, b);
        if d < bestd {
            bestd = d;
            best = i;
        }
    }
    best
}

fn nearest_rgb(r: i32, g: i32, b: i32) -> i32 {
    nearest_in(r, g, b, 256)
}

/// Best colour a terminal with `colors` colours can show for `c`.
pub fn nearest(c: i32, colors: u32) -> u8 {
    if colors >= 256 {
        return c.min(255) as u8;
    }
    if colors < 2 {
        return 0;
    }
    let (r, g, b) = xterm_rgb(c.min(255));
    nearest_in(r, g, b, colors as i32) as u8
}

/// Lighten (k > 1) or darken (k < 1) an xterm colour, for the wall face
/// shades of the isometric views.
pub fn shade(c: i32, k: f64) -> i32 {
    let (r, g, b) = xterm_rgb(c.min(255));
    let f = |v: i32| (((v as f64) * k + 0.5) as i32).min(255);
    nearest_rgb(f(r), f(g), f(b))
}

static NAMES: &[(&str, i32)] = &[
    ("black", 0),
    ("maroon", 1),
    ("green", 2),
    ("olive", 10),
    ("blue", 4),
    ("purple", 129),
    ("cyan", 6),
    ("white", 7),
    ("silver", 251),
    ("gray", 245),
    ("grey", 245),
    ("darkgray", 243),
    ("lightgray", 252),
    ("red", 1),
    ("brightred", 9),
    ("lightred", 9),
    ("crimson", 196),
    ("brightgreen", 10),
    ("lightgreen", 10),
    ("lime", 46),
    ("seagreen", 29),
    ("forest", 28),
    ("brightyellow", 11),
    ("lightyellow", 11),
    ("gold", 220),
    ("khaki", 222),
    ("brightblue", 12),
    ("lightblue", 12),
    ("navy", 4),
    ("slate", 67),
    ("sky", 117),
    ("azure", 45),
    ("brightmagenta", 13),
    ("lightmagenta", 13),
    ("magenta", 5),
    ("pink", 218),
    ("violet", 177),
    ("indigo", 54),
    ("orchid", 175),
    ("brightcyan", 14),
    ("lightcyan", 14),
    ("teal", 30),
    ("turquoise", 44),
    ("aquamarine", 122),
    ("brightwhite", 15),
    ("lightwhite", 15),
    ("orange", 214),
    ("coral", 203),
    ("salmon", 209),
    ("brown", 94),
    ("tan", 180),
    ("wheat", 223),
    ("ivory", 231),
    ("darkgreen", 22),
    ("darkblue", 18),
    ("darkred", 88),
    ("darkmagenta", 90),
    ("darkcyan", 23),
    ("darkyellow", 58),
];

/// Lower-case and drop '-', '_' and ' ', so "Dark-Blue" == "darkblue".
pub fn normalize(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '-' && c != '_' && c != ' ')
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// A colour name, an xterm index 0-255, or hex (#rrggbb, 0xrrggbb,
/// rrggbb).  Returns the xterm index.
pub fn parse_color(s: &str) -> Option<i32> {
    let buf = s.trim_matches(|c| c == ' ' || c == '\t');
    if buf.is_empty() {
        return None;
    }

    let hex = buf
        .strip_prefix('#')
        .or_else(|| buf.strip_prefix("0x"))
        .or_else(|| buf.strip_prefix("0X"))
        .unwrap_or(buf);
    if hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        let p = |i: usize| i32::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
        return Some(nearest_rgb(p(0), p(2), p(4)));
    }

    if let Some(v) = parse_plain_decimal(buf)
        && (0..=255).contains(&v)
    {
        return Some(v as i32);
    }

    let norm = normalize(buf);
    NAMES.iter().find(|(n, _)| *n == norm).map(|&(_, i)| i)
}

/// strtol(s, &end, 10) with the whole string consumed.
fn parse_plain_decimal(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let digits = t.strip_prefix(['+', '-']).unwrap_or(t);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

/// wall, path, head, trail1, trail2, unvisited, endpoint,
/// wall-top, wall-left, wall-right (the isometric face shades), bg.
pub type Palette = [i32; ROLE_COUNT];

static THEMES: &[(&str, Palette)] = &[
    ("default", [27, 251, 51, 50, 44, 238, 226, 39, 27, 18, 0]),
    ("forest", [29, 108, 118, 78, 43, 22, 226, 35, 29, 23, 0]),
    ("ocean", [33, 109, 123, 45, 38, 17, 122, 45, 33, 24, 0]),
    (
        "amber",
        [166, 143, 226, 214, 172, 58, 231, 215, 166, 130, 0],
    ),
    (
        "mono",
        [245, 242, 255, 251, 248, 236, 231, 253, 245, 238, 0],
    ),
];

pub fn theme_lookup(name: &str) -> Option<Palette> {
    let norm = normalize(name);
    THEMES
        .iter()
        .find(|(n, _)| normalize(n) == norm)
        .map(|&(_, p)| p)
}

/// Theme colours with any `--colors` overrides applied.
pub fn palette(theme: &str, colors: Option<&str>) -> Palette {
    let mut cols = theme_lookup(theme)
        .or_else(|| theme_lookup("default"))
        .unwrap_or([245; ROLE_COUNT]);

    if let Some(list) = colors.filter(|s| !s.is_empty()) {
        for (idx, tok) in list.split(',').take(ROLE_COUNT).enumerate() {
            if let Some(v) = parse_color(tok) {
                cols[idx] = v;
            }
        }
        // A custom wall colour brings its own face shades.
        if let Some(v) = list.split(',').next().and_then(parse_color) {
            cols[Role::WTop as usize] = shade(v, 1.45);
            cols[Role::WLeft as usize] = v;
            cols[Role::WRight as usize] = shade(v, 0.6);
        }
    }
    cols
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Auto,
    Texture,
    Ascii,
    Unicode,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Top,
    Iso,
    Oblique,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsoStyle {
    Auto,
    Ascii,
    Box,
    Block,
}

pub fn parse_style(name: &str) -> Option<Style> {
    if name.is_empty() {
        return Some(Style::Auto);
    }
    match normalize(name).as_str() {
        "auto" => Some(Style::Auto),
        "texture" => Some(Style::Texture),
        "ascii" => Some(Style::Ascii),
        "unicode" | "utf8" => Some(Style::Unicode),
        "block" | "solid" => Some(Style::Block),
        _ => None,
    }
}

pub fn parse_view(name: &str) -> Option<View> {
    if name.is_empty() {
        return Some(View::Top);
    }
    match normalize(name).as_str() {
        "top" | "down" => Some(View::Top),
        "iso" | "isometric" => Some(View::Iso),
        "oblique" => Some(View::Oblique),
        _ => None,
    }
}

pub fn parse_iso_style(name: &str) -> Option<IsoStyle> {
    if name.is_empty() {
        return Some(IsoStyle::Auto);
    }
    match normalize(name).as_str() {
        "auto" => Some(IsoStyle::Auto),
        "ascii" => Some(IsoStyle::Ascii),
        "box" | "unicode" => Some(IsoStyle::Box),
        "block" | "solid" => Some(IsoStyle::Block),
        _ => None,
    }
}

/// Whether the locale can carry UTF-8, from LC_ALL / LC_CTYPE / LANG.
/// An unset or "C"/"POSIX" locale is treated as UTF-8, like the original
/// which switched those to a UTF-8 locale at start-up.
pub fn locale_is_utf8() -> bool {
    let var = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty());
    match var {
        None => true,
        Some(v) if v == "C" || v == "POSIX" => true,
        Some(v) => {
            let l = v.to_ascii_lowercase();
            l.contains("utf-8") || l.contains("utf8")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_colors() {
        assert_eq!(parse_color("red"), Some(1));
        assert_eq!(parse_color(" Dark-Blue "), Some(18));
        assert_eq!(parse_color("200"), Some(200));
        assert_eq!(parse_color("256"), None);
        assert_eq!(parse_color("#ff0000"), Some(9));
        assert_eq!(parse_color("0x00ff00"), Some(10));
        assert_eq!(parse_color("nope"), None);
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn nearest_maps_down() {
        assert_eq!(nearest(196, 256), 196);
        assert_eq!(nearest(196, 16), 9);
        assert_eq!(nearest(27, 8), 6);
    }
}
