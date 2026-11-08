//! Command line parsing, help text and save/load files.
//!
//! The parser follows getopt_long semantics so the command line behaves
//! exactly as before: bundled short flags (`-le`), attached or separate
//! arguments (`-t0.5`, `-t 0.5`, `--time=0.5`, `--time 0.5`), unique
//! prefixes of long options (`--brai=20`) and `--` to end options.

use std::fmt::Write as _;
use std::fs;

use crate::algos;
use crate::theme::{self, IsoStyle, Style, View};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Bits recording which settings the user really typed, so a `--load`
/// file never overrides the command line.
mod x {
    pub const TIME: u32 = 1 << 0;
    pub const WAIT: u32 = 1 << 1;
    pub const TYPE: u32 = 1 << 2;
    pub const SEED: u32 = 1 << 3;
    pub const STYLE: u32 = 1 << 4;
    pub const THEME: u32 = 1 << 5;
    pub const WW: u32 = 1 << 6;
    pub const CORR: u32 = 1 << 7;
    pub const BRAID: u32 = 1 << 8;
    pub const STRAT: u32 = 1 << 9;
    pub const SIZE: u32 = 1 << 10;
    pub const COLORS: u32 = 1 << 11;
    pub const MSG: u32 = 1 << 12;
    pub const ENDPOINTS: u32 = 1 << 13;
    pub const FILL: u32 = 1 << 14;
    pub const VIEW: u32 = 1 << 15;
    pub const WALLH: u32 = 1 << 16;
    pub const ROTATE: u32 = 1 << 17;
    pub const SHADE: u32 = 1 << 18;
    pub const NOFLOOR: u32 = 1 << 19;
    pub const ISOSTYLE: u32 = 1 << 20;
}

#[derive(Clone, Debug)]
pub struct Options {
    pub live: bool,
    pub infinite: bool,
    pub screensaver: bool,
    pub print: bool,
    pub endpoints: bool,
    pub time: f64,
    pub wait: f64,
    pub algo: String,
    pub seed: u64,
    pub seed_set: bool,
    pub message: Option<String>,
    pub colors: Option<String>,
    pub style_arg: Option<String>,
    pub theme: String,
    pub strategy: String,
    pub wall_width: i32,
    pub corridor: i32,
    pub braid: i32,
    /// % of the space the maze may claim.
    pub fill: i32,
    /// -1 => derive from the terminal.
    pub size_w: i32,
    pub size_h: i32,
    /// top | iso | oblique, as typed.
    pub view: String,
    pub iso_style_arg: Option<String>,
    pub wall_height: i32,
    /// 0..3, 90 degree steps.
    pub rotate: i32,
    pub shade: bool,
    pub floor: bool,
    pub save: Option<String>,
    pub load: Option<String>,
    pub help: bool,
    pub version: bool,
    /// Resolved from the strings above in `main`.
    pub style: Style,
    pub view_mode: View,
    pub iso_style: IsoStyle,
    explicit: u32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            live: false,
            infinite: false,
            screensaver: false,
            print: false,
            endpoints: false,
            time: 0.01,
            wait: 1.0,
            algo: "backtracker".into(),
            seed: 0,
            seed_set: false,
            message: None,
            colors: None,
            style_arg: None,
            theme: "default".into(),
            strategy: "mix".into(),
            wall_width: 1,
            corridor: 2,
            braid: 0,
            fill: 67,
            size_w: -1,
            size_h: -1,
            view: "top".into(),
            iso_style_arg: None,
            wall_height: 2,
            rotate: 0,
            shade: true,
            floor: true,
            save: None,
            load: None,
            help: false,
            version: false,
            style: Style::Auto,
            view_mode: View::Top,
            iso_style: IsoStyle::Auto,
            explicit: 0,
        }
    }
}

const STRATEGIES: [&str; 4] = ["newest", "oldest", "random", "mix"];

pub const TYPE_LIST: &str = "backtracker prim kruskal wilson aldous-broder eller \
growing-tree division sidewinder binary-tree hunt-and-kill random";

pub fn print_version() {
    println!("mazecarve {VERSION}");
}

pub fn print_help() {
    print!(
        "\
mazecarve {VERSION} - a terminal maze generator

usage: mazecarve [options]

modes
  -l, --live              animate the maze being carved
  -i, --infinite          keep generating new mazes forever
  -S, --screensaver       live + infinite, quits on any keypress
  -p, --print             print the finished maze and exit (no TUI)

timing
  -t, --time=TIME         seconds between animation steps (default 0.01)
  -w, --wait=TIME         seconds to pause between mazes (default 1.0)
  -s, --seed=INT          seed, so the same maze comes back

appearance
  -T, --type=TYPE         maze algorithm (default backtracker)
      --style=STYLE       dots, texture, ascii, unicode or block (default dots)
      --theme=THEME       default, forest, ocean, amber or mono
  -c, --colors=LIST       comma list: wall,path,head,trail1,trail2,unvisited,endpoint
  -W, --wall-width=INT    wall thickness in characters (default 1)
  -C, --corridor=INT      corridor width in characters (default 2)
      --fill=PCT          share of the terminal the maze may use (default 67)
      --size=WxH          fixed maze size in cells
  -m, --message=STR       show a message below the maze, like cbonsai
  -e, --endpoints         mark an entrance and exit

views
  -V, --view=VIEW         top (default), iso or oblique
  -H, --wall-height=INT   wall height in rows for iso/oblique (default 2)
  -R, --rotate=INT        which corner faces you: 0, 1, 2 or 3 (default 0)
      --shade=BOOL        shade wall faces light/medium/dark (default on)
      --no-floor          draw only walls, no floor tiles
      --iso-style=STYLE   iso glyphs: ascii, box or block (default auto)

tweaks
  -b, --braid=INT         percentage of dead ends to remove (0-100)
      --strategy=STRAT    growing tree: newest, oldest, random or mix

files
  -f, --save=FILE         save the seed and settings to FILE
  -L, --load=FILE         load a saved maze configuration

  -v, --version           print the version and exit
  -h, --help              print this help and exit

maze types
  backtracker prim kruskal wilson aldous-broder eller growing-tree
  division sidewinder binary-tree hunt-and-kill random

keys
  q, ESC, Ctrl-C          quit (any key quits in screensaver mode)

examples
  mazecarve -l                        watch a maze carve itself
  mazecarve -l -T wilson --theme=ocean
  mazecarve -l -S                     screensaver
  mazecarve -p --size=40x20 > maze.txt
  mazecarve -l -s 42 -b 40 -e -m \"hello\"
  mazecarve -V iso -l -H 3            isometric view, taller walls
  mazecarve -p -V iso -R 1 > maze.txt
"
    );
}

// ------------------------------------------------------------------
// C-compatible number parsing, so values are read exactly as before.
// ------------------------------------------------------------------

/// strtol/strtoull with base 0 (0x hex, leading-0 octal, else decimal).
/// Returns (negative, magnitude, bytes consumed); consumed == 0 when no
/// number was found.
fn c_strtonum(s: &str) -> (bool, u128, usize) {
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || (b'\t'..=b'\r').contains(&b[i])) {
        i += 1;
    }
    let mut neg = false;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        neg = b[i] == b'-';
        i += 1;
    }
    let (base, start) = if i + 1 < b.len()
        && b[i] == b'0'
        && (b[i + 1] | 0x20) == b'x'
        && b.get(i + 2).is_some_and(|c| c.is_ascii_hexdigit())
    {
        (16, i + 2)
    } else if i < b.len() && b[i] == b'0' {
        (8, i)
    } else {
        (10, i)
    };
    let mut j = start;
    let mut v: u128 = 0;
    while j < b.len() {
        let Some(d) = (b[j] as char).to_digit(base) else {
            break;
        };
        v = v.saturating_mul(base as u128).saturating_add(d as u128);
        j += 1;
    }
    if j == start {
        return (false, 0, 0);
    }
    (neg, v, j)
}

/// strtol(s, &end, 0) requiring the whole string to be consumed.
fn parse_long_full(s: &str) -> Option<i64> {
    let (neg, v, n) = c_strtonum(s);
    if n == 0 || n != s.len() {
        return None;
    }
    Some(to_long(neg, v))
}

fn to_long(neg: bool, v: u128) -> i64 {
    let v = v.min(i64::MAX as u128 + neg as u128);
    if neg {
        (v as i128).wrapping_neg() as i64
    } else {
        v as i64
    }
}

/// strtol(s, NULL, 0): 0 when there is no number.
fn parse_long_prefix(s: &str) -> i64 {
    let (neg, v, _) = c_strtonum(s);
    to_long(neg, v)
}

/// strtoull(s, ..., 0): negative input wraps, as in C.
fn parse_ull(s: &str) -> (u64, usize) {
    let (neg, v, n) = c_strtonum(s);
    let v = v.min(u64::MAX as u128) as u64;
    (if neg { v.wrapping_neg() } else { v }, n)
}

/// atoi: decimal prefix, 0 when there is none.
fn atoi(s: &str) -> i64 {
    let t = s.trim_start();
    let (neg, rest) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let v: i64 = digits.parse().unwrap_or(0);
    if neg { -v } else { v }
}

/// atof: the longest numeric prefix, 0.0 when there is none.
fn atof(s: &str) -> f64 {
    let t = s.trim_start();
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let lower = t[i..].to_ascii_lowercase();
    for word in ["infinity", "inf", "nan"] {
        if lower.starts_with(word) {
            return t[..i + word.len()].parse().unwrap_or(0.0);
        }
    }
    let ds = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == ds || (i == ds + 1 && b[ds] == b'.') {
        return 0.0;
    }
    let mantissa_end = i;
    if i < b.len() && (b[i] | 0x20) == b'e' {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let es = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > es {
            i = j;
        }
    }
    t[..i]
        .parse()
        .or_else(|_| t[..mantissa_end].parse())
        .unwrap_or(0.0)
}

/// sscanf(s, "%dx%d", &w, &h): returns how many fields matched.
fn scan_size(s: &str) -> (i32, i32, i32) {
    let int = |s: &str| -> Option<(i32, usize)> {
        let t = s.trim_start();
        let skip = s.len() - t.len();
        let b = t.as_bytes();
        let mut i = 0;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let ds = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == ds {
            return None;
        }
        let v: i64 = t[..i].parse().ok()?;
        Some((v.clamp(i32::MIN as i64, i32::MAX as i64) as i32, skip + i))
    };
    let Some((w, n)) = int(s) else {
        return (0, 0, 0);
    };
    match s[n..].strip_prefix('x').and_then(int) {
        Some((h, _)) => (2, w, h),
        None => (1, w, 0),
    }
}

fn parse_bool(arg: &str) -> Option<bool> {
    match arg.to_ascii_lowercase().as_str() {
        "yes" | "true" | "on" | "1" => Some(true),
        "no" | "false" | "off" | "0" => Some(false),
        _ => None,
    }
}

// ------------------------------------------------------------------
// getopt_long
// ------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Opt {
    Live,
    Infinite,
    Time,
    Wait,
    Screensaver,
    Type,
    Seed,
    Message,
    Colors,
    WallWidth,
    Corridor,
    Braid,
    Endpoints,
    Print,
    Save,
    Load,
    View,
    WallHeight,
    Rotate,
    Version,
    Help,
    Style,
    Theme,
    Size,
    Strategy,
    Fill,
    Shade,
    NoShade,
    NoFloor,
    IsoStyle,
}

/// (long name, takes an argument, option)
const LONG: &[(&str, bool, Opt)] = &[
    ("live", false, Opt::Live),
    ("infinite", false, Opt::Infinite),
    ("time", true, Opt::Time),
    ("wait", true, Opt::Wait),
    ("screensaver", false, Opt::Screensaver),
    ("type", true, Opt::Type),
    ("seed", true, Opt::Seed),
    ("message", true, Opt::Message),
    ("colors", true, Opt::Colors),
    ("wall-width", true, Opt::WallWidth),
    ("corridor", true, Opt::Corridor),
    ("braid", true, Opt::Braid),
    ("endpoints", false, Opt::Endpoints),
    ("print", false, Opt::Print),
    ("save", true, Opt::Save),
    ("load", true, Opt::Load),
    ("view", true, Opt::View),
    ("wall-height", true, Opt::WallHeight),
    ("rotate", true, Opt::Rotate),
    ("version", false, Opt::Version),
    ("help", false, Opt::Help),
    ("style", true, Opt::Style),
    ("theme", true, Opt::Theme),
    ("size", true, Opt::Size),
    ("strategy", true, Opt::Strategy),
    ("fill", true, Opt::Fill),
    ("shade", true, Opt::Shade),
    ("no-shade", false, Opt::NoShade),
    ("no-floor", false, Opt::NoFloor),
    ("iso-style", true, Opt::IsoStyle),
];

fn short(c: char) -> Option<(bool, Opt)> {
    Some(match c {
        'l' => (false, Opt::Live),
        'i' => (false, Opt::Infinite),
        't' => (true, Opt::Time),
        'w' => (true, Opt::Wait),
        'S' => (false, Opt::Screensaver),
        'T' => (true, Opt::Type),
        's' => (true, Opt::Seed),
        'm' => (true, Opt::Message),
        'c' => (true, Opt::Colors),
        'W' => (true, Opt::WallWidth),
        'C' => (true, Opt::Corridor),
        'b' => (true, Opt::Braid),
        'e' => (false, Opt::Endpoints),
        'p' => (false, Opt::Print),
        'f' => (true, Opt::Save),
        'L' => (true, Opt::Load),
        'V' => (true, Opt::View),
        'H' => (true, Opt::WallHeight),
        'R' => (true, Opt::Rotate),
        'v' => (false, Opt::Version),
        'h' => (false, Opt::Help),
        _ => return None,
    })
}

fn usage_error(msg: String) -> String {
    format!("{msg}\ntry 'mazecarve --help'")
}

/// Splits argv into (option, argument) pairs, getopt_long style.
fn tokenize(args: &[String]) -> Result<Vec<(Opt, Option<String>)>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        i += 1;
        if a == "--" {
            if i < args.len() {
                return Err(usage_error(format!("unexpected argument -- '{}'", args[i])));
            }
            break;
        }
        if let Some(body) = a.strip_prefix("--") {
            let (name, inline) = match body.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (body, None),
            };
            let exact = LONG.iter().find(|(n, _, _)| *n == name);
            let entry = match exact {
                Some(e) => e,
                None => {
                    let hits: Vec<_> = LONG
                        .iter()
                        .filter(|(n, _, _)| n.starts_with(name))
                        .collect();
                    match hits.as_slice() {
                        [one] => *one,
                        [] => return Err(usage_error(format!("invalid option -- '{a}'"))),
                        _ => return Err(usage_error(format!("option '--{name}' is ambiguous"))),
                    }
                }
            };
            let (lname, takes, opt) = *entry;
            if takes {
                let val = match inline {
                    Some(v) => v,
                    None if i < args.len() => {
                        i += 1;
                        args[i - 1].clone()
                    }
                    None => {
                        return Err(usage_error(format!(
                            "option '--{lname}' requires an argument"
                        )));
                    }
                };
                out.push((opt, Some(val)));
            } else {
                if inline.is_some() {
                    return Err(usage_error(format!(
                        "option '--{lname}' doesn't allow an argument"
                    )));
                }
                out.push((opt, None));
            }
            continue;
        }
        if let Some(body) = a.strip_prefix('-').filter(|b| !b.is_empty()) {
            for (k, c) in body.char_indices() {
                let Some((takes, opt)) = short(c) else {
                    return Err(usage_error(format!("invalid option -- '{c}'")));
                };
                if !takes {
                    out.push((opt, None));
                    continue;
                }
                let rest = &body[k + c.len_utf8()..];
                let val = if !rest.is_empty() {
                    rest.to_string()
                } else if i < args.len() {
                    i += 1;
                    args[i - 1].clone()
                } else {
                    return Err(usage_error(format!("option requires an argument -- '{c}'")));
                };
                out.push((opt, Some(val)));
                break;
            }
            continue;
        }
        return Err(usage_error(format!("unexpected argument -- '{a}'")));
    }
    Ok(out)
}

impl Options {
    /// Parses argv (without the program name).  Errors carry the full
    /// message to print, without the "mazecarve: " prefix.
    pub fn parse(args: &[String]) -> Result<Options, String> {
        let mut o = Options::default();
        for (opt, arg) in tokenize(args)? {
            o.apply(opt, arg.as_deref().unwrap_or(""))?;
        }
        if let Some(path) = o.load.clone() {
            o.load_file(&path)
                .map_err(|_| format!("cannot read {path}"))?;
        }
        if algos::find(&o.algo).is_none() && o.algo != "random" {
            return Err(format!(
                "unknown type '{}'\ntry one of: {TYPE_LIST}",
                o.algo
            ));
        }
        Ok(o)
    }

    fn apply(&mut self, opt: Opt, arg: &str) -> Result<(), String> {
        let int = |what: &str| parse_long_full(arg).ok_or_else(|| format!("{what}: not a number"));
        match opt {
            Opt::Live => self.live = true,
            Opt::Infinite => self.infinite = true,
            Opt::Screensaver => {
                self.screensaver = true;
                self.live = true;
                self.infinite = true;
            }
            Opt::Print => self.print = true,
            Opt::Endpoints => {
                self.endpoints = true;
                self.explicit |= x::ENDPOINTS;
            }
            Opt::Version => self.version = true,
            Opt::Help => self.help = true,
            Opt::Time => {
                let v = atof(arg);
                if v < 0.0 {
                    return Err(format!("time must not be negative: {arg}"));
                }
                self.time = v;
                self.explicit |= x::TIME;
            }
            Opt::Wait => {
                let v = atof(arg);
                if v < 0.0 {
                    return Err(format!("wait must not be negative: {arg}"));
                }
                self.wait = v;
                self.explicit |= x::WAIT;
            }
            Opt::Type => {
                self.algo = arg.to_string();
                self.explicit |= x::TYPE;
            }
            Opt::Seed => {
                let (v, n) = parse_ull(arg);
                if n == 0 || n != arg.len() {
                    return Err(format!("bad seed: {arg}"));
                }
                self.seed = v;
                self.seed_set = true;
                self.explicit |= x::SEED;
            }
            Opt::Message => {
                self.message = Some(arg.to_string());
                self.explicit |= x::MSG;
            }
            Opt::Colors => {
                self.colors = Some(arg.to_string());
                self.explicit |= x::COLORS;
            }
            Opt::WallWidth => {
                let v = int("wall width")?;
                if v < 1 {
                    return Err(format!("wall width must be >= 1: {arg}"));
                }
                self.wall_width = v as i32;
                self.explicit |= x::WW;
            }
            Opt::Corridor => {
                let v = int("corridor width")?;
                if v < 1 {
                    return Err(format!("corridor width must be >= 1: {arg}"));
                }
                self.corridor = v as i32;
                self.explicit |= x::CORR;
            }
            Opt::Braid => {
                let v = int("braid")?;
                if !(0..=100).contains(&v) {
                    return Err(format!("braid must be 0-100: {arg}"));
                }
                self.braid = v as i32;
                self.explicit |= x::BRAID;
            }
            Opt::Save => self.save = Some(arg.to_string()),
            Opt::Load => self.load = Some(arg.to_string()),
            Opt::View => {
                if theme::parse_view(arg).is_none() {
                    return Err(format!("unknown view: {arg} (try top, iso, oblique)"));
                }
                self.view = arg.to_string();
                self.explicit |= x::VIEW;
            }
            Opt::WallHeight => {
                let v = int("wall height")?;
                if v < 1 {
                    return Err(format!("wall height must be >= 1: {arg}"));
                }
                if v > 100 {
                    return Err(format!("wall height must be <= 100: {arg}"));
                }
                self.wall_height = v as i32;
                self.explicit |= x::WALLH;
            }
            Opt::Rotate => {
                let v = int("rotate")?;
                if !(0..=3).contains(&v) {
                    return Err(format!("rotate must be 0-3: {arg}"));
                }
                self.rotate = v as i32;
                self.explicit |= x::ROTATE;
            }
            Opt::Shade => {
                self.shade =
                    parse_bool(arg).ok_or_else(|| format!("shade needs yes or no: {arg}"))?;
                self.explicit |= x::SHADE;
            }
            Opt::NoShade => {
                self.shade = false;
                self.explicit |= x::SHADE;
            }
            Opt::NoFloor => {
                self.floor = false;
                self.explicit |= x::NOFLOOR;
            }
            Opt::IsoStyle => {
                if theme::parse_iso_style(arg).is_none() {
                    return Err(format!("unknown iso style: {arg} (try ascii, box, block)"));
                }
                self.iso_style_arg = Some(arg.to_string());
                self.explicit |= x::ISOSTYLE;
            }
            Opt::Style => {
                if theme::parse_style(arg).is_none() {
                    return Err(format!(
                        "unknown style: {arg} (try dots, texture, ascii, unicode, block)"
                    ));
                }
                self.style_arg = Some(arg.to_string());
                self.explicit |= x::STYLE;
            }
            Opt::Theme => {
                if theme::theme_lookup(arg).is_none() {
                    return Err(format!(
                        "unknown theme: {arg} (try default, forest, ocean, amber, mono)"
                    ));
                }
                self.theme = arg.to_string();
                self.explicit |= x::THEME;
            }
            Opt::Size => {
                let (n, w, h) = scan_size(arg);
                if n < 1 || w < 1 || h < 0 {
                    return Err(format!("size must look like 40x20: {arg}"));
                }
                self.size_w = w;
                self.size_h = if h > 0 { h } else { -1 };
                self.explicit |= x::SIZE;
            }
            Opt::Strategy => {
                if !STRATEGIES.contains(&arg) {
                    return Err(format!(
                        "unknown strategy: {arg} (newest, oldest, random, mix)"
                    ));
                }
                self.strategy = arg.to_string();
                self.explicit |= x::STRAT;
            }
            Opt::Fill => {
                let v = int("fill")?;
                if !(1..=100).contains(&v) {
                    return Err(format!("fill must be 1-100: {arg}"));
                }
                self.fill = v as i32;
                self.explicit |= x::FILL;
            }
        }
        Ok(())
    }

    // --------------------------------------------------------------
    // Save / load
    // --------------------------------------------------------------

    fn load_kv(&mut self, key: &str, val: &str) {
        let ex = self.explicit;
        let free = |bit: u32| ex & bit == 0;
        match key {
            "seed" if free(x::SEED) => {
                self.seed = parse_ull(val).0;
                self.seed_set = true;
            }
            "type" if free(x::TYPE) && (algos::find(val).is_some() || val == "random") => {
                self.algo = val.to_string();
            }
            "time" if free(x::TIME) => self.time = atof(val),
            "wait" if free(x::WAIT) => self.wait = atof(val),
            "style" if free(x::STYLE) && theme::parse_style(val).is_some() => {
                self.style_arg = Some(val.to_string());
            }
            "theme" if free(x::THEME) && theme::theme_lookup(val).is_some() => {
                self.theme = val.to_string();
            }
            "wall-width" if free(x::WW) => {
                let v = parse_long_prefix(val);
                if v >= 1 {
                    self.wall_width = v as i32;
                }
            }
            "corridor" if free(x::CORR) => {
                let v = parse_long_prefix(val);
                if v >= 1 {
                    self.corridor = v as i32;
                }
            }
            "fill" if free(x::FILL) => {
                let v = parse_long_prefix(val);
                if (1..=100).contains(&v) {
                    self.fill = v as i32;
                }
            }
            "braid" if free(x::BRAID) => {
                let v = parse_long_prefix(val);
                if (0..=100).contains(&v) {
                    self.braid = v as i32;
                }
            }
            "strategy" if free(x::STRAT) && STRATEGIES.contains(&val) => {
                self.strategy = val.to_string();
            }
            "size" if free(x::SIZE) => {
                let (n, w, h) = scan_size(val);
                if n >= 1 && w >= 1 {
                    self.size_w = w;
                    self.size_h = if h > 0 { h } else { -1 };
                }
            }
            "colors" if free(x::COLORS) && !val.is_empty() => self.colors = Some(val.to_string()),
            "message" if free(x::MSG) && !val.is_empty() => self.message = Some(val.to_string()),
            "endpoints" if free(x::ENDPOINTS) => self.endpoints = atoi(val) != 0,
            "view" if free(x::VIEW) && theme::parse_view(val).is_some() => {
                self.view = val.to_string();
            }
            "wall-height" if free(x::WALLH) => {
                let v = parse_long_prefix(val);
                if (1..=100).contains(&v) {
                    self.wall_height = v as i32;
                }
            }
            "rotate" if free(x::ROTATE) => {
                let v = parse_long_prefix(val);
                if (0..=3).contains(&v) {
                    self.rotate = v as i32;
                }
            }
            "shade" if free(x::SHADE) => self.shade = atoi(val) != 0,
            "floor" if free(x::NOFLOOR) => self.floor = atoi(val) != 0,
            "iso-style" if free(x::ISOSTYLE) && theme::parse_iso_style(val).is_some() => {
                self.iso_style_arg = Some(val.to_string());
            }
            _ => {}
        }
    }

    pub fn load_file(&mut self, path: &str) -> std::io::Result<()> {
        let bytes = fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        for line in text.lines() {
            // Lines without '=' are the header or comments.
            if let Some((k, v)) = line.split_once('=') {
                self.load_kv(k, v);
            }
        }
        Ok(())
    }

    pub fn save_text(&self, seed: u64) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "mazecarve {VERSION} seed and settings");
        let _ = writeln!(s, "seed={seed}");
        let _ = writeln!(s, "type={}", self.algo);
        let _ = writeln!(s, "time={:.4}", self.time);
        let _ = writeln!(s, "wait={:.4}", self.wait);
        let _ = writeln!(s, "style={}", self.style_arg.as_deref().unwrap_or("auto"));
        let _ = writeln!(s, "theme={}", self.theme);
        let _ = writeln!(s, "wall-width={}", self.wall_width);
        let _ = writeln!(s, "corridor={}", self.corridor);
        let _ = writeln!(s, "fill={}", self.fill);
        let _ = writeln!(s, "braid={}", self.braid);
        let _ = writeln!(s, "strategy={}", self.strategy);
        let _ = writeln!(s, "size={}x{}", self.size_w.max(0), self.size_h.max(0));
        let _ = writeln!(s, "colors={}", self.colors.as_deref().unwrap_or(""));
        let _ = writeln!(s, "message={}", self.message.as_deref().unwrap_or(""));
        let _ = writeln!(s, "endpoints={}", self.endpoints as i32);
        let _ = writeln!(s, "view={}", self.view);
        let _ = writeln!(s, "wall-height={}", self.wall_height);
        let _ = writeln!(s, "rotate={}", self.rotate);
        let _ = writeln!(s, "shade={}", self.shade as i32);
        let _ = writeln!(s, "floor={}", self.floor as i32);
        let _ = writeln!(
            s,
            "iso-style={}",
            self.iso_style_arg.as_deref().unwrap_or("auto")
        );
        s
    }

    pub fn save_file(&self, path: &str, seed: u64) -> std::io::Result<()> {
        fs::write(path, self.save_text(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn short_long_and_bundled() {
        let o = parse(&[
            "-le",
            "-t0.5",
            "--seed",
            "0x10",
            "--brai=20",
            "-T",
            "Wilson",
        ])
        .unwrap();
        assert!(o.live && o.endpoints);
        assert_eq!(o.time, 0.5);
        assert_eq!(o.seed, 16);
        assert_eq!(o.braid, 20);
        assert_eq!(o.algo, "Wilson");
        let o = parse(&["-S", "--no-shade", "--size=40", "--shade=on"]).unwrap();
        assert!(o.screensaver && o.live && o.infinite && o.shade);
        assert_eq!((o.size_w, o.size_h), (40, -1));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse(&["--wall=2"]).is_err()); // ambiguous
        assert!(parse(&["-x"]).is_err());
        assert!(parse(&["stray"]).is_err());
        assert!(parse(&["-b", "101"]).is_err());
        assert!(parse(&["-T", "nope"]).is_err());
        assert!(parse(&["--size=x3"]).is_err());
        assert!(parse(&["-t"]).is_err());
    }

    #[test]
    fn save_load_round_trip() {
        let o = parse(&[
            "-s",
            "42",
            "-T",
            "prim",
            "-b",
            "30",
            "-V",
            "iso",
            "-H",
            "3",
            "--no-floor",
            "-m",
            "hi",
        ])
        .unwrap();
        let dir = std::env::temp_dir().join(format!("mazecarve-test-{}", std::process::id()));
        let path = dir.to_string_lossy().to_string();
        o.save_file(&path, 42).unwrap();
        let back = parse(&["-L", &path, "-b", "5"]).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(back.seed, 42);
        assert_eq!(back.algo, "prim");
        assert_eq!(back.braid, 5, "command line wins over the file");
        assert_eq!(back.view, "iso");
        assert_eq!(back.wall_height, 3);
        assert!(!back.floor);
        assert_eq!(back.message.as_deref(), Some("hi"));
    }

    #[test]
    fn c_number_parsing() {
        assert_eq!(parse_long_full("010"), Some(8));
        assert_eq!(parse_long_full("0x1f"), Some(31));
        assert_eq!(parse_long_full("12a"), None);
        assert_eq!(atof("0.25s"), 0.25);
        assert_eq!(atof("abc"), 0.0);
        assert_eq!(scan_size("40x20"), (2, 40, 20));
        assert_eq!(scan_size("40X20"), (1, 40, 0));
    }
}
