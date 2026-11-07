//! Eller's algorithm: builds the maze one row at a time, a scanline
//! sweeping down the grid.

use super::Algo;
use crate::maze::{DIR_E, DIR_S, Maze};
use crate::rng::Rng;

#[derive(PartialEq, Eq)]
enum Phase {
    Merge,
    Down,
    Advance,
    Done,
}

pub struct Eller {
    row: i32,
    phase: Phase,
    /// Cursor inside `Phase::Merge`.
    col: usize,
    /// Cursor inside `Phase::Down`.
    seti: usize,
    next_id: i32,
    /// Set id of each cell in the current row (0 = none yet).
    sid: Vec<i32>,
    /// Id inherited by the next row.
    next_sid: Vec<i32>,
    /// Distinct ids present in this row.
    sets: Vec<i32>,
}

impl Eller {
    fn ensure_id(&mut self, c: usize) {
        if self.sid[c] == 0 {
            self.sid[c] = self.next_id;
            self.next_id += 1;
        }
    }

    fn relabel(&mut self, from: i32, to: i32) {
        if from == to {
            return;
        }
        for s in &mut self.sid {
            if *s == from {
                *s = to;
            }
        }
    }

    fn build_sets(&mut self) {
        self.sets.clear();
        for &s in &self.sid {
            if !self.sets.contains(&s) {
                self.sets.push(s);
            }
        }
    }
}

fn mark_row(m: &mut Maze, r: i32) {
    for c in 0..m.cols {
        m.mark(r, c);
    }
    m.set_head(r, 0);
}

pub fn create(m: &mut Maze, _rng: &mut Rng, _strategy: &str) -> Box<dyn Algo> {
    let cols = m.cols as usize;
    let mut s = Eller {
        row: 0,
        phase: Phase::Merge,
        col: 0,
        seti: 0,
        next_id: 1,
        sid: vec![0; cols],
        next_sid: vec![0; cols],
        sets: Vec::with_capacity(cols),
    };
    m.reset(true);
    s.ensure_id(0);
    mark_row(m, 0);
    Box::new(s)
}

impl Algo for Eller {
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool {
        let cols = m.cols as usize;
        match self.phase {
            Phase::Merge => {
                let last_row = self.row == m.rows - 1;
                if self.col + 1 >= cols {
                    if last_row {
                        self.phase = Phase::Done;
                    } else {
                        self.build_sets();
                        self.seti = 0;
                        self.phase = Phase::Down;
                    }
                    return false;
                }
                let c = self.col;
                self.ensure_id(c);
                self.ensure_id(c + 1);
                if self.sid[c] != self.sid[c + 1] {
                    let join = last_row || rng.double() < 0.5;
                    if join {
                        m.carve(self.row, c as i32, DIR_E);
                        self.relabel(self.sid[c + 1], self.sid[c]);
                    }
                }
                self.col += 1;
                m.set_head(self.row, self.col as i32);
                false
            }
            Phase::Down => {
                if self.seti >= self.sets.len() {
                    self.phase = Phase::Advance;
                    return false;
                }
                let set = self.sets[self.seti];
                let mut chosen: Option<usize> = None;
                let mut members = 0u32;
                let mut dropped = 0;
                for i in 0..cols {
                    if self.sid[i] != set {
                        continue;
                    }
                    members += 1;
                    if rng.double() < 0.5 {
                        m.carve(self.row, i as i32, DIR_S);
                        self.next_sid[i] = set;
                        dropped += 1;
                        chosen = Some(i);
                    }
                }
                // Every set must reach the row below.
                if dropped == 0 && members > 0 {
                    let mut pick = rng.below(members) as i64;
                    for i in 0..cols {
                        if self.sid[i] != set {
                            continue;
                        }
                        if pick == 0 {
                            m.carve(self.row, i as i32, DIR_S);
                            self.next_sid[i] = set;
                            chosen = Some(i);
                            break;
                        }
                        pick -= 1;
                    }
                }
                if let Some(c) = chosen {
                    m.set_head(self.row, c as i32);
                }
                self.seti += 1;
                false
            }
            Phase::Advance => {
                self.row += 1;
                if self.row >= m.rows {
                    self.phase = Phase::Done;
                    return false;
                }
                for i in 0..cols {
                    self.sid[i] = self.next_sid[i];
                    self.next_sid[i] = 0;
                }
                self.phase = Phase::Merge;
                self.col = 0;
                self.ensure_id(0);
                mark_row(m, self.row);
                false
            }
            Phase::Done => {
                m.set_head(-1, -1);
                true
            }
        }
    }
}
