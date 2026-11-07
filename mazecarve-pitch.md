# mazecarve (née cmaze): the original pitch

> This is the design pitch the project started from, when it was `cmaze`,
> written in C + ncurses. mazecarve is the Rust + ratatui port of that
> program; the concept, flags and maze types below still apply.

## One-liner
**cmaze** is to mazes what **cbonsai** is to bonsai trees: a small, pretty, terminal-native generator that grows a maze on screen. Nobody plays it and nothing solves it. It's meant to be watched, screenshotted, or left running as a screensaver.

## Concept
The user launches `cmaze` and a maze appears in the terminal. In live mode (`-l`) you watch it carve itself out step by step. In infinite mode (`-i`) it finishes, pauses, clears, and starts a new one with a different seed or algorithm. The project is purely visual: no player, no pathfinding, no goal, no solver. The appeal is the generation process itself and the look of the result.

## Tech constraints
- Language: C (C99 or newer)
- UI: ncurses only (color pairs, `mvaddch`, `timeout`/`nodelay` for non-blocking input)
- Build: a simple Makefile with `make` and `make install`, plus a man page
- No dependencies beyond ncurses and libc
- Adapts to terminal size and handles `SIGWINCH` (resize) gracefully
- Clean exit on `q` or Ctrl-C, restoring the terminal state

## CLI options (modeled on cbonsai)

| Flag | Long form | Description |
|---|---|---|
| `-l` | `--live` | Animate the maze generation step by step |
| `-i` | `--infinite` | Keep generating new mazes forever |
| `-t TIME` | `--time=TIME` | Delay between animation steps (live mode) |
| `-w TIME` | `--wait=TIME` | Pause between mazes in infinite mode |
| `-S` | `--screensaver` | Live + infinite, quits on any keypress |
| `-T TYPE` | `--type=TYPE` | Maze algorithm (see below) |
| `-s SEED` | `--seed=INT` | Seed for reproducible mazes |
| `-m STR` | `--message=STR` | Show a message in a box, like cbonsai |
| `-c LIST` | `--colors=LIST` | Custom color list (walls, paths, generator head, etc.) |
| `-W INT` | `--wall-width=INT` | Wall thickness / cell size in characters |
| `-C INT` | `--corridor=INT` | Corridor width |
| `--fill=PCT` | `--fill=PCT` | Share of the terminal the maze fills (default 67) |
| `-b INT` | `--braid=INT` | Percentage of dead ends removed, to create loops |
| `-e` | `--endpoints` | Mark an entrance and exit (purely decorative) |
| `-p` | `--print` | Print the final maze to stdout and exit (no ncurses session) |
| `-f FILE` | `--save=FILE` | Save the seed and settings to a file |
| `-L FILE` | `--load=FILE` | Load a saved maze config |
| `-v` | `--version` | Print version |
| `-h` | `--help` | Print help |

## Maze types (`-T`)
Each type is a different generation algorithm, so each looks and animates differently:

1. **backtracker**: Recursive backtracker (DFS). Long, winding corridors with few branches. The default.
2. **prim**: Randomized Prim's. Many short dead ends, a bushy look, grows outward like a stain.
3. **kruskal**: Randomized Kruskal's. Uniform texture, with many small regions merging.
4. **wilson**: Wilson's algorithm. Unbiased, with loop-erased random walks that look great live.
5. **aldous-broder**: Aldous-Broder. Unbiased but slow, a hypnotic random-walk animation.
6. **eller**: Eller's algorithm. Builds row by row, a scanline effect.
7. **growing-tree**: Growing Tree with a selectable strategy (newest, random, mix).
8. **division**: Recursive division. Walls get added instead of carved, producing long straight walls and a "room" feel.
9. **sidewinder**: Sidewinder. Strong horizontal bias, with an open top row.
10. **binary-tree**: Binary Tree. Diagonal bias, with two clear open edges.
11. **hunt-and-kill**: Hunt-and-Kill. Like the backtracker but with a visible scanning phase.
12. **random**: Pick a random type each time (great with `-i`).

**Stretch goals:**
- Different cell shapes: hexagonal, triangular, or circular (polar) mazes.
- Braid/loop mazes (`-b`), and weave mazes with over/under crossings.

## Rendering ideas
- **Wall styles:** ASCII (`+ - |`), box-drawing Unicode (`┌ ─ ┐ │ ┘ └ ├ ┤ ┬ ┴ ┼`), or solid blocks (`█`). Selectable with a `--style` flag.
- **Live mode visuals:** a highlighted "head" showing the active cell, visited and unvisited cells in different colors, and a short color fade trail behind the head.
- **Color themes:** presets (`--theme=forest|ocean|amber|mono`) as well as custom colors via `-c`.
- **Centered layout:** the maze is sized to the terminal and centered, with optional fixed dimensions via `--size=WxH`.
- **Message box:** same idea as cbonsai's `-m`, drawn in a corner.

## Architecture suggestions
- `main.c`: argument parsing (`getopt_long`), setup, main loop
- `grid.c/h`: a cell grid with wall bitflags (N/E/S/W) and a visited flag
- `algos/*.c`: one file per algorithm, each exposing a **step function** (`step(grid, state)` returns done or not), so live mode can draw after every step and non-live mode can just loop until done
- `render.c/h`: converts the grid to screen characters according to the style and theme
- `theme.c/h`: color setup
- `rng.c/h`: seeded RNG for reproducibility

The key design choice is the step-based algorithm interface. It makes `-l`, `-t`, `-i`, and `-p` all work with the same code.

## Non-goals
- No player movement, no solving, no pathfinding visualization
- No scoring, levels, or game logic
- No mouse support, no GUI

## Deliverables for the developing AI
1. Full C source with the Makefile
2. A README with install steps, usage examples, and a few ASCII screenshots
3. A man page (`cmaze.6`)
4. Start with the MVP: `backtracker`, `prim`, and `kruskal`, plus `-l`, `-i`, `-t`, `-w`, `-s`, `-p`, `-h`, `-v`. Then add the remaining algorithms, styles, and themes.

## Name and tone
The name is `cmaze`, a nod to cbonsai. The goal is a small, zen, screensaver-like tool that is satisfying to watch.
