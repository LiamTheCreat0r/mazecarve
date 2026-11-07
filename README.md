# mazecarve

A terminal maze generator in Rust + [ratatui](https://ratatui.rs), meant to be
watched rather than played. Carve a maze one cell at a time, in color, at
whatever speed you like — or print one to a file. Walls default to a seeded
ASCII texture, and the same maze can be drawn top-down, isometric, or oblique:

```
                     __
                   ______
                 ____\/____
               ____\.||:/____
             ____\.|.__:|:/____
           ____\.|.______:|:/____
         ____\.|.____\/____:|:/____
       ____\.|.____\.||:/____:|:/____
     ____\.|.____\.|.__:|:/____:|:/____
   ______|. /____|.____\ :|____\ :|:/____
 ____\/____ |:/______\.| ____\.| __:|:/____
/____||:/__\ :|____\.|.______|. /____:|____\
|:/____:|:.| ____\.|.____\/____ |:/______\.|
|:|:/____:. /__\.|.____\.||____\ :|____\.|.|
  |:|:/____ |:.|.____\.|.____\.| ____\.|.|
    |:|:/____:.____\.|.____\.|.____\.|.|
      |:|:/______\.|.____\.|.____\.|.|
        |:|:/____|. /__\.|.____\.|.|
          |:|:/____ |:.|.____\.|.|
            |:|:/____:.____\.|.|
              |:|:/______\.|.|
                |:|:/__\.|.|
                  |:|:.|.|
                    |:.|
```

## Features

- **11 maze algorithms**, plus `random` to pick one per maze
- **Three views** — top-down (default), isometric (`-V iso`) and oblique
  (`-V oblique`); the same seed draws the same maze in every view
- **Textured walls (default)** — walls composed of seeded ASCII symbols,
  grouped by stroke direction, so the maze reads clearly but looks hand-typed
- **Three more wall styles** — plain ASCII, Unicode box-drawing, or solid blocks
  (`--style=ascii|unicode|block`)
- **Live mode** — watch the maze carve itself, cell by cell
- **Infinite / screensaver mode** — endless mazes, the screensaver quits on any key
- **Themes and custom colors** — `default`, `forest`, `ocean`, `amber`, `mono`, or your own palette
- **Braiding** — remove dead ends for a loopy maze
- **Endpoints** — carve an entrance and exit
- **Messages** — a framed message below the maze, like cbonsai
- **Deterministic seeds** — the same seed always draws the same maze
- **Save / load** — persist a seed plus its settings to a file
- **Resize-safe** — the maze re-lays out (or is rebuilt) when the terminal changes size
- **Clean exits** — the terminal is restored on `q`, Ctrl-C, `SIGTERM` or a panic

## Building

Requires a recent stable Rust toolchain (edition 2024).

```sh
cargo install --path .        # installs the mazecarve binary into ~/.cargo/bin
cargo build --release         # or just build ./target/release/mazecarve
```

The man page is `mazecarve.6`; to install it:

```sh
install -m 644 mazecarve.6 /usr/local/share/man/man6/
```

## Usage

```
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
      --style=STYLE       texture, ascii, unicode or block (default texture)
      --theme=THEME       default, forest, ocean, amber or mono
  -c, --colors=LIST       comma list: wall,path,head,trail1,trail2,unvisited,endpoint
  -W, --wall-width=INT    wall thickness in characters (default 1)
  -C, --corridor=INT      corridor width in characters (default 2)
      --fill=PCT          share of the terminal the maze may use (default 67)
      --size=WxH          fixed maze size in cells
  -m, --message=STR       show a message below the maze, like cbonsai
  -e, --endpoints         mark an entrance and exit

tweaks
  -b, --braid=INT         percentage of dead ends to remove (0-100)
      --strategy=STRAT    growing tree: newest, oldest, random or mix

views
  -V, --view=VIEW         top (default), iso or oblique
  -H, --wall-height=INT   wall height in rows for iso/oblique (default 2)
  -R, --rotate=INT        which corner faces you: 0, 1, 2 or 3 (default 0)
      --shade=BOOL        shade wall faces light/medium/dark (default on)
      --no-floor          draw only walls, no floor tiles
      --iso-style=STYLE   iso glyphs: ascii, box or block (default auto)

files
  -f, --save=FILE         save the seed and settings to FILE
  -L, --load=FILE         load a saved maze configuration

  -v, --version           print the version and exit
  -h, --help              print this help and exit
```

### Keys

| Key | Action |
| --- | --- |
| `q`, `ESC`, `Ctrl-C` | quit (any key quits in screensaver mode) |

The terminal is restored cleanly on exit, whatever way you quit.

### Examples

```sh
mazecarve -l                          # watch a maze carve itself
mazecarve -l -T wilson --theme=ocean  # Wilson's algorithm, ocean colors
mazecarve -l -S                       # screensaver
mazecarve -l -i -w 2                  # new maze every 2 seconds, forever
mazecarve -p --size=40x20 > maze.txt  # print a maze to a file
mazecarve -l -s 42 -b 40 -e -m "hello" # seeded, braided, with endpoints + message
mazecarve -V iso -l -H 3              # isometric view with taller walls
mazecarve -p -V iso -R 1 -s 42        # same maze as above, rotated, printed
```

## Maze types

| Type | Notes |
| --- | --- |
| `backtracker` | depth-first; long windy corridors (default) |
| `prim` | randomized Prim; short dead ends, rustic look |
| `kruskal` | randomized Kruskal; shuffle-and-merge |
| `wilson` | Wilson's uniform algorithm; unbiased, pretty |
| `aldous-broder` | pure random walk; uniform but slow to converge |
| `eller` | row-by-row, low memory |
| `growing-tree` | blend of DFS and BFS via `--strategy` |
| `division` | recursive division; long straight walls |
| `sidewinder` | run-and-choose, biased corridors |
| `binary-tree` | fastest, strong northeast bias |
| `hunt-and-kill` | walk until stuck, then hunt for unvisited cells |
| `random` | pick a different algorithm each maze |

## Wall styles

| Style | Look |
| --- | --- |
| `texture` (default) | random ASCII symbols per wall cell, chosen from a hash of position + seed; horizontal runs use `-=~_:.`, vertical runs use `|!:'`, corners and tees use `+#*x@%`, crossings use `+#x*` |
| `ascii` | plain `\|`, `-`, `+` |
| `unicode` | box-drawing strokes `─│┌┐└┘├┤┬┴┼` |
| `block` | solid `█` walls |

```sh
mazecarve -p --style=unicode    # the classic box-drawing maze
mazecarve -l --style=block      # chunky block walls
```

The texture is deterministic: the same `--seed` always produces the same
glyphs, and each maze in infinite mode gets its own texture.

## Views

`-V` picks how the maze is drawn. Nothing about generation changes — the
same seed always produces the same maze, whatever the view.

| View | Look |
| --- | --- |
| `top` (default) | the classic flat plan |
| `iso` | 2:1 dimetric isometric; walls become blocks with a top and two side faces |
| `oblique` | front-facing with a diagonal depth offset; a cheaper, simpler 3D look |

The isometric view is a parallel projection (no perspective), approximated
with the classic 2:1 pixel-art slope so it lands cleanly on a character grid.
Tiles are drawn back to front, so walls in front hide what is behind them.
Mazes are auto-sized to fit the terminal and centered; live carving looks
like corridors being dug out of a solid block.

```sh
mazecarve -V iso -l              # watch it carve in 3D
mazecarve -V iso -H 3 -R 1       # taller walls, rotated 90 degrees
mazecarve -V oblique -l          # simpler oblique blocks
mazecarve -p -V iso > maze.txt   # print the isometric maze (plain ASCII)
```

`--iso-style` chooses the glyph set: `ascii` (`/ \ _ |`), `box` (box drawing
and diagonals) or `block` (`█▓▒░`). The live view defaults to `box` on a
UTF-8 terminal, printed output always defaults to `ascii`. Wall faces are
shaded via the theme's wall colour (top lightest, left medium, right
darkest); `--no-shade` turns that off, and 8/16-color terminals fall back
to denser glyphs instead of colors.

## Colors

A theme sets the whole palette; `-c` overrides individual roles:

```sh
mazecarve -l -c "#ff0000,#00ff00,#ffffff"   # hex colors
mazecarve -l -c "196,46,231"                # xterm-256 indices
```

Roles, in order: `wall,path,head,trail1,trail2,unvisited,endpoint`.
`head` is the carving front, `trail1`/`trail2` are the receding glow behind it.
Colors are remapped to the nearest available on 8/16-color terminals. The
color depth comes from `TERM` and `COLORTERM`, and `NO_COLOR` turns colors off.

## Save files

`-f` writes the current seed and settings as `key=value` lines; `-L` reads them
back (flags given on the command line still win). Files saved by the old C
`cmaze` load unchanged:

```
mazecarve 2.0.0 seed and settings
seed=42
type=backtracker
size=40x20
theme=default
braid=40
endpoints=1
view=iso
wall-height=3
rotate=1
```

## Project layout

```
src/main.rs           entry point, print mode, generation driver
src/tui.rs            live view: ratatui loop, input, resize, signals
src/options.rs        getopt_long-style parser, help/version, save/load
src/maze.rs           grid + wall bookkeeping, braiding, endpoints
src/algos/            the 11 algorithms, one file each, plus the registry
src/canvas.rs         off-screen character/role buffer
src/tiles.rs          maze -> tile grid expansion, rotation math
src/render/top.rs     top-down view: strokes, texture, blocks
src/render/iso.rs     isometric + oblique renderers (projection, faces)
src/render/mod.rs     shared render state, message box, printed output
src/layout.rs         sizing and centring on the terminal
src/theme.rs          color names, xterm-256 mapping, themes, styles
src/rng.rs            splitmix64 PRNG, identical on every platform
```

## History

mazecarve started life as `cmaze`, written in C + ncurses. The Rust version
is a step-for-step port: for the same seed and flags it prints byte-for-byte
the same maze as `cmaze` 1.1.0 did.

## See also

- `mazecarve(6)` — the man page
- `mazecarve-pitch.md` — the original design pitch
