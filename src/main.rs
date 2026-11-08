//! mazecarve - a terminal maze generator, meant to be watched rather
//! than played.

mod algos;
mod canvas;
mod layout;
mod maze;
mod options;
mod render;
mod rng;
mod theme;
mod tiles;
mod tui;

use std::io::{IsTerminal, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use algos::Algo;
use maze::{Maze, TRAIL_2};
use options::Options;
use rng::Rng;
use theme::{IsoStyle, Style, View};

/// Guard against an algorithm that never reports "done".
pub const STEP_CAP: u64 = 50_000_000;

/// A maze being generated.
pub struct Gen {
    pub maze: Maze,
    pub algo: Box<dyn Algo>,
    pub name: &'static str,
}

impl Gen {
    pub fn start(o: &Options, rows: i32, cols: i32, rng: &mut Rng) -> Gen {
        let mut maze = Maze::new(rows, cols);
        let info = if o.algo == "random" {
            algos::pick(rng)
        } else {
            algos::find(&o.algo).expect("validated while parsing")
        };
        let algo = (info.create)(&mut maze, rng, &o.strategy);
        Gen {
            maze,
            algo,
            name: info.name,
        }
    }

    /// One step; true when the maze is finished.
    pub fn step(&mut self, rng: &mut Rng) -> bool {
        let done = self.algo.step(&mut self.maze, rng);
        self.maze.tick = self.maze.tick.wrapping_add(1);
        done
    }

    /// Decorate, then settle into a clean, uniform maze for the final
    /// screenshot.
    pub fn finish(&mut self, o: &Options, rng: &mut Rng) {
        if o.braid > 0 {
            self.maze.braid(o.braid, rng);
        }
        if o.endpoints {
            self.maze.endpoints(rng);
        }
        self.maze.set_head(-1, -1);
        self.maze.tick = self.maze.tick.wrapping_add(TRAIL_2 as u32 + 1);
    }
}

pub fn default_seed() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (now.as_secs())
        .wrapping_mul(1_000_000)
        .wrapping_add(now.subsec_micros() as u64)
        .wrapping_add((std::process::id() as u64).wrapping_mul(1_000_003))
}

/// Terminal size (rows, cols), 24x80 when it cannot be read.
pub fn term_size() -> (i32, i32) {
    match ratatui::crossterm::terminal::size() {
        Ok((c, r)) if r > 1 && c > 1 => (r as i32, c as i32),
        _ => (24, 80),
    }
}

fn run_print(o: &mut Options) -> ExitCode {
    let (tr, tc) = term_size();
    let (rows, cols) = layout::autosize(o, tr, tc);

    let seed = if o.seed_set { o.seed } else { default_seed() };
    o.seed = seed; // the texture is hashed from this
    let mut rng = Rng::new(seed);

    let mut g = Gen::start(o, rows, cols, &mut rng);
    let mut steps = 0u64;
    while !g.step(&mut rng) {
        steps += 1;
        if steps > STEP_CAP {
            eprintln!("mazecarve: {} did not finish, giving up", g.name);
            break;
        }
    }
    g.finish(o, &mut rng);

    let text = render::print_text(&g.maze, o);
    let mut out = std::io::stdout().lock();
    if out
        .write_all(text.as_bytes())
        .and_then(|_| out.flush())
        .is_err()
    {
        return ExitCode::from(1);
    }

    if let Some(path) = &o.save
        && o.save_file(path, seed).is_err()
    {
        eprintln!("mazecarve: cannot write {path}");
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = match Options::parse(&args) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("mazecarve: {msg}");
            return ExitCode::from(2);
        }
    };

    if o.help {
        options::print_help();
        return ExitCode::SUCCESS;
    }
    if o.version {
        options::print_version();
        return ExitCode::SUCCESS;
    }

    if let Some(s) = &o.style_arg {
        o.style = theme::parse_style(s).unwrap_or(Style::Auto);
    }
    let utf8 = theme::locale_is_utf8();
    if o.style == Style::Auto {
        o.style = Style::Dots;
    }
    if !utf8 && matches!(o.style, Style::Unicode | Style::Block) {
        eprintln!("mazecarve: no UTF-8 locale, falling back to dot walls");
        o.style = Style::Dots;
    }

    o.view_mode = theme::parse_view(&o.view).unwrap_or(View::Top);
    o.iso_style = o
        .iso_style_arg
        .as_deref()
        .and_then(theme::parse_iso_style)
        .unwrap_or(IsoStyle::Auto);
    if o.iso_style == IsoStyle::Auto {
        // The live view may use box drawing; printed output stays plain
        // ASCII so it can be pasted anywhere.
        o.iso_style = if !o.print && utf8 {
            IsoStyle::Box
        } else {
            IsoStyle::Ascii
        };
    } else if !utf8 && matches!(o.iso_style, IsoStyle::Box | IsoStyle::Block) {
        eprintln!("mazecarve: no UTF-8 locale, falling back to ascii glyphs");
        o.iso_style = IsoStyle::Ascii;
    }

    if o.print {
        return run_print(&mut o);
    }
    if !std::io::stdout().is_terminal() {
        eprintln!("mazecarve: stdout is not a terminal");
        eprintln!("mazecarve: use 'mazecarve -p' to print a maze instead");
        return ExitCode::from(1);
    }
    tui::run(&mut o)
}
