//! The algorithm registry: name lookup and random selection.

use crate::maze::Maze;
use crate::rng::Rng;

mod aldous_broder;
mod backtracker;
mod binary_tree;
mod division;
mod eller;
mod growing_tree;
mod hunt_kill;
mod kruskal;
mod prim;
mod sidewinder;
mod wilson;

/// A maze generator, advanced one animation step at a time.
pub trait Algo {
    /// One animation step; returns true when the maze is finished.
    fn step(&mut self, m: &mut Maze, rng: &mut Rng) -> bool;
}

/// Resets `m` for this algorithm and returns its state.
/// `strategy` is only read by growing-tree.
pub type CreateFn = fn(&mut Maze, &mut Rng, &str) -> Box<dyn Algo>;

pub struct AlgoInfo {
    pub name: &'static str,
    #[allow(dead_code)]
    pub desc: &'static str,
    pub create: CreateFn,
}

pub static REGISTRY: &[AlgoInfo] = &[
    AlgoInfo {
        name: "backtracker",
        desc: "recursive backtracker (DFS): long winding corridors",
        create: backtracker::create,
    },
    AlgoInfo {
        name: "prim",
        desc: "randomized Prim's: bushy, many short dead ends",
        create: prim::create,
    },
    AlgoInfo {
        name: "kruskal",
        desc: "randomized Kruskal's: uniform texture, regions merging",
        create: kruskal::create,
    },
    AlgoInfo {
        name: "wilson",
        desc: "Wilson's: unbiased loop-erased random walks",
        create: wilson::create,
    },
    AlgoInfo {
        name: "aldous-broder",
        desc: "Aldous-Broder: unbiased random walk, hypnotic and slow",
        create: aldous_broder::create,
    },
    AlgoInfo {
        name: "eller",
        desc: "Eller's: builds row by row, a scanline effect",
        create: eller::create,
    },
    AlgoInfo {
        name: "growing-tree",
        desc: "Growing Tree: selectable strategy mixes depth and breadth",
        create: growing_tree::create,
    },
    AlgoInfo {
        name: "division",
        desc: "recursive division: walls appear, long straight lines",
        create: division::create,
    },
    AlgoInfo {
        name: "sidewinder",
        desc: "Sidewinder: horizontal runs with a strong eastward bias",
        create: sidewinder::create,
    },
    AlgoInfo {
        name: "binary-tree",
        desc: "Binary Tree: diagonal bias with two clear open edges",
        create: binary_tree::create,
    },
    AlgoInfo {
        name: "hunt-and-kill",
        desc: "Hunt-and-Kill: like the backtracker with a visible scanning phase",
        create: hunt_kill::create,
    },
];

/// Case/dash-insensitive: `aldous_broder`, `Aldous-Broder` and
/// `aldous-broder` all match.
fn name_eq(a: &str, b: &str) -> bool {
    let norm = |ch: char| {
        if ch == '_' {
            '-'
        } else {
            ch.to_ascii_lowercase()
        }
    };
    a.len() == b.len() && a.chars().map(norm).eq(b.chars().map(norm))
}

pub fn find(name: &str) -> Option<&'static AlgoInfo> {
    REGISTRY.iter().find(|a| name_eq(a.name, name))
}

/// Never returns the "random" alias.
pub fn pick(rng: &mut Rng) -> &'static AlgoInfo {
    &REGISTRY[rng.below(REGISTRY.len() as u32) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::wall_bit;

    #[test]
    fn find_normalises_names() {
        assert_eq!(find("Aldous_Broder").unwrap().name, "aldous-broder");
        assert_eq!(find("HUNT-AND-KILL").unwrap().name, "hunt-and-kill");
        assert!(find("random").is_none());
        assert!(find("prims").is_none());
    }

    /// Every generator must yield a perfect maze: a spanning tree, so
    /// every cell is reachable and there are exactly cells-1 passages.
    #[test]
    fn every_algo_makes_a_perfect_maze() {
        for info in REGISTRY {
            for strategy in ["newest", "oldest", "random", "mix"] {
                for (rows, cols, seed) in
                    [(1, 1, 1), (1, 9, 2), (9, 1, 3), (13, 21, 4), (20, 20, 5)]
                {
                    let mut m = Maze::new(rows, cols);
                    let mut rng = Rng::new(seed);
                    let mut algo = (info.create)(&mut m, &mut rng, strategy);
                    let mut steps = 0u64;
                    while !algo.step(&mut m, &mut rng) {
                        m.tick = m.tick.wrapping_add(1);
                        steps += 1;
                        assert!(steps < 50_000_000, "{} did not finish", info.name);
                    }
                    let mut passages = 0;
                    let mut seen = vec![false; (rows * cols) as usize];
                    let mut todo = vec![(0, 0)];
                    seen[0] = true;
                    while let Some((r, c)) = todo.pop() {
                        for d in 0..4 {
                            if m.walls[m.idx(r, c)] & wall_bit(d) != 0 {
                                continue;
                            }
                            if let Some((nr, nc)) = m.nb(r, c, d) {
                                passages += 1;
                                let ni = m.idx(nr, nc);
                                if !seen[ni] {
                                    seen[ni] = true;
                                    todo.push((nr, nc));
                                }
                            }
                        }
                    }
                    assert!(
                        seen.iter().all(|&s| s),
                        "{} left cells unreachable",
                        info.name
                    );
                    assert_eq!(passages / 2, rows * cols - 1, "{} made loops", info.name);
                }
            }
        }
    }
}
