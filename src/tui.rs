//! The live view: carve mazes on screen with ratatui, one step at a time.

use std::io;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::layout::{self, Layout};
use crate::maze::{Maze, Role};
use crate::options::Options;
use crate::render::{self, Render};
use crate::rng::Rng;
use crate::theme::View;
use crate::{Gen, STEP_CAP, default_seed};

/// How often (in steps) a non-animated carve checks for input.
const POLL_EVERY: u64 = 4096;

/// Restores the terminal however we leave, including early returns.
struct TermGuard;

impl Drop for TermGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

/// Colours the terminal can show: 0 (none), 8, 16 or 256.
fn detect_colors() -> u32 {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    if !env("NO_COLOR").is_empty() {
        return 0;
    }
    let term = env("TERM");
    if term.is_empty() || term == "dumb" {
        return 0;
    }
    let colorterm = env("COLORTERM");
    if colorterm == "truecolor" || colorterm == "24bit" || term.contains("256") {
        256
    } else if term.contains("16color") {
        16
    } else {
        8
    }
}

/// An xterm colour index as a ratatui colour.  The first 16 use the
/// basic ANSI codes so 8 and 16 colour terminals understand them.
fn color(c: u8) -> Color {
    match c {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        n => Color::Indexed(n),
    }
}

/// One frame: the maze and the message box.
struct Frame<'a> {
    maze: Option<&'a Maze>,
    o: &'a Options,
    r: &'a Render,
    l: &'a Layout,
}

impl Frame<'_> {
    fn role_style(&self, role: Role, solid: bool) -> Style {
        if !self.r.use_color {
            return Style::default();
        }
        let c = color(self.r.col[role as usize]);
        if solid && self.r.colors >= 16 {
            // Same foreground and background, so a space paints a slab.
            if role == Role::Bg {
                Style::default()
            } else {
                Style::default().fg(c).bg(c)
            }
        } else {
            Style::default().fg(c)
        }
    }
}

fn put(buf: &mut Buffer, x: i32, y: i32, ch: char, style: Style) {
    if x < 0 || y < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }
    if let Some(cell) = buf.cell_mut((x as u16, y as u16)) {
        cell.set_char(ch).set_style(style);
    }
}

impl Widget for Frame<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let l = self.l;
        let (scr_w, scr_h) = (area.width as i32, area.height as i32);

        if let Some(m) = self.maze {
            let cv = render::build(l.view, l.cw, l.ch, m, self.o, self.r);
            let solid = l.view != View::Top;
            // The bottom band belongs to the message box.  Clip against
            // the layout and the real screen, so a stale layout after a
            // resize never draws past the edge.
            let lim = (l.tr - l.msg_h).min(scr_h);
            for y in 0..cv.h {
                let sy = l.oy + y;
                if sy >= lim {
                    break;
                }
                for x in 0..cv.w {
                    let sx = l.ox + x;
                    if sx >= l.tc || sx >= scr_w {
                        break;
                    }
                    let i = cv.at(x, y);
                    put(buf, sx, sy, cv.ch[i], self.role_style(cv.role[i], solid));
                }
            }
        }

        if l.msg_h > 0
            && let Some(mb) = render::msg_prepare(self.o, l.tc)
        {
            let border = self.role_style(Role::Head, false);
            let text = self.role_style(Role::Endpoint, false);
            let (w, h) = (l.msg_w, l.msg_h);
            for (j, row) in render::msg_rows(&mb, self.r.style).iter().enumerate() {
                let j = j as i32;
                if j >= h {
                    break;
                }
                for (i, &ch) in row.iter().enumerate() {
                    let i = i as i32;
                    if i >= w {
                        break;
                    }
                    let edge = j == 0 || j == h - 1 || i == 0 || i == w - 1;
                    put(
                        buf,
                        l.msg_x + i,
                        l.msg_y + j,
                        ch,
                        if edge { border } else { text },
                    );
                }
            }
        }
    }
}

/// What the user (or the terminal) asked for.
enum Input {
    Quit,
    Resize(u16, u16),
}

struct Ui {
    term: DefaultTerminal,
    quit_signal: Arc<AtomicBool>,
    screensaver: bool,
    /// Terminal changed shape: rebuild the maze.
    restart: bool,
    /// Layout moved: redraw.
    changed: bool,
}

impl Ui {
    fn draw(&mut self, maze: Option<&Maze>, o: &Options, r: &Render, l: &Layout) -> io::Result<()> {
        self.term
            .draw(|f| f.render_widget(Frame { maze, o, r, l }, f.area()))?;
        Ok(())
    }

    /// Wait up to `timeout` for input that matters.
    fn poll(&mut self, timeout: Duration) -> io::Result<Option<Input>> {
        let deadline = Instant::now() + timeout;
        loop {
            if self.quit_signal.load(Ordering::Relaxed) {
                return Ok(Some(Input::Quit));
            }
            // Wake at least every 50ms so signals are noticed.
            let left = deadline.saturating_duration_since(Instant::now());
            if event::poll(left.min(Duration::from_millis(50)))? {
                match event::read()? {
                    Event::Resize(c, r) => return Ok(Some(Input::Resize(r, c))),
                    Event::Key(k) if k.kind != KeyEventKind::Release => {
                        let ctrl_c = k.code == KeyCode::Char('c')
                            && k.modifiers.contains(KeyModifiers::CONTROL);
                        let quit = matches!(k.code, KeyCode::Char('q' | 'Q') | KeyCode::Esc);
                        if self.screensaver || quit || ctrl_c {
                            return Ok(Some(Input::Quit));
                        }
                    }
                    _ => {}
                }
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
        }
    }

    /// The terminal is now tr x tc: relayout, or ask for a rebuild.
    fn resized(&mut self, o: &Options, l: &mut Layout, tr: u16, tc: u16) {
        let (tr, tc) = (tr.max(1) as i32, tc.max(1) as i32);
        let (rows, cols) = layout::autosize(o, tr, tc);
        let nl = layout::place(rows, cols, o, tr, tc);
        if rows == l.rows && cols == l.cols {
            *l = nl;
            self.changed = true;
        } else if nl.cw <= tc && nl.ch <= tr {
            self.restart = true; // rebuild at the new size
        } else {
            *l = nl; // oversized on purpose: draw clipped
            self.changed = true;
        }
    }

    /// Show the finished maze for `seconds` (None = until quit or resize).
    /// Returns true when the user asked to quit.
    fn pause(
        &mut self,
        seconds: Option<f64>,
        m: &Maze,
        o: &Options,
        r: &Render,
        l: &mut Layout,
    ) -> io::Result<bool> {
        let deadline = seconds.map(|s| Instant::now() + Duration::from_secs_f64(s));
        loop {
            let wait = match deadline {
                Some(d) => d.saturating_duration_since(Instant::now()),
                None => Duration::from_millis(250),
            };
            match self.poll(wait)? {
                Some(Input::Quit) => return Ok(true),
                Some(Input::Resize(tr, tc)) => self.resized(o, l, tr, tc),
                None => {}
            }
            if self.restart {
                return Ok(false);
            }
            if self.changed {
                self.changed = false;
                self.draw(Some(m), o, r, l)?;
            }
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return Ok(false);
            }
        }
    }
}

enum Outcome {
    Quit,
    GaveUp(&'static str),
}

pub fn run(o: &mut Options) -> ExitCode {
    let quit_signal = Arc::new(AtomicBool::new(false));
    for sig in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        let _ = signal_hook::flag::register(sig, Arc::clone(&quit_signal));
    }

    let colors = detect_colors();
    let mut r = Render::for_terminal(o, colors >= 8, colors);

    let term = match ratatui::try_init() {
        Ok(t) => t,
        Err(e) => {
            ratatui::restore();
            eprintln!("mazecarve: cannot start the terminal: {e}");
            return ExitCode::from(1);
        }
    };
    let guard = TermGuard;
    let mut ui = Ui {
        term,
        quit_signal,
        screensaver: o.screensaver,
        restart: false,
        changed: false,
    };

    let mut save_error = None;
    let result = live(&mut ui, o, &mut r, &mut save_error);
    drop(ui);
    drop(guard);

    if let Some(path) = save_error {
        eprintln!("mazecarve: cannot write {path}");
    }
    match result {
        Ok(Outcome::Quit) => ExitCode::SUCCESS,
        Ok(Outcome::GaveUp(name)) => {
            eprintln!("mazecarve: {name} did not finish, giving up");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mazecarve: {e}");
            ExitCode::from(1)
        }
    }
}

fn live(
    ui: &mut Ui,
    o: &Options,
    r: &mut Render,
    save_error: &mut Option<String>,
) -> io::Result<Outcome> {
    let base = if o.seed_set { o.seed } else { default_seed() };
    let mut maze_no: u64 = 0;
    let mut saved = false;
    let tick = Duration::from_secs_f64(o.time);

    loop {
        let size = ui.term.size()?;
        let (tr, tc) = (size.height.max(1) as i32, size.width.max(1) as i32);
        let (rows, cols) = layout::autosize(o, tr, tc);
        let mut l = layout::place(rows, cols, o, tr, tc);

        let seed = base.wrapping_add(maze_no);
        let mut rng = Rng::new(seed);
        r.tex_seed = seed as u32; // the wall texture varies per maze
        ui.restart = false;
        ui.changed = false;

        let mut g = Gen::start(o, l.rows, l.cols, &mut rng);
        ui.draw(Some(&g.maze), o, r, &l)?;

        // ---- carve ----
        let mut steps = 0u64;
        loop {
            let started = Instant::now();
            let done = g.step(&mut rng);
            steps += 1;

            let input = if o.live {
                ui.draw(Some(&g.maze), o, r, &l)?;
                ui.poll(tick.saturating_sub(started.elapsed()))?
            } else if steps.is_multiple_of(POLL_EVERY) {
                ui.poll(Duration::ZERO)?
            } else {
                None
            };
            match input {
                Some(Input::Quit) => return Ok(Outcome::Quit),
                Some(Input::Resize(tr, tc)) => ui.resized(o, &mut l, tr, tc),
                None => {}
            }
            if ui.changed && o.live {
                ui.changed = false;
            }
            if ui.restart {
                break;
            }
            if steps > STEP_CAP {
                return Ok(Outcome::GaveUp(g.name));
            }
            if done {
                break;
            }
        }
        if ui.restart {
            // New size, new maze.
            maze_no += 1;
            continue;
        }

        // ---- settle and show ----
        g.finish(o, &mut rng);
        ui.changed = false;
        ui.draw(Some(&g.maze), o, r, &l)?;

        if let Some(path) = &o.save
            && !saved
        {
            if o.save_file(path, seed).is_err() {
                *save_error = Some(path.clone());
            }
            saved = true;
        }

        if !o.infinite {
            if ui.pause(None, &g.maze, o, r, &mut l)? {
                return Ok(Outcome::Quit);
            }
            // Terminal changed shape: rebuild at the new size with the
            // same seed, so the maze changes shape but not identity.
            continue;
        }

        if ui.pause(Some(o.wait), &g.maze, o, r, &mut l)? {
            return Ok(Outcome::Quit);
        }
        maze_no += 1;
    }
}
