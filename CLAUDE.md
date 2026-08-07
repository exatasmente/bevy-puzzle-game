# CLAUDE.md

Guidance for AI assistants working in this repository.

## What this project is

A color-matching puzzle game ("Color Puzzle") built with **Bevy 0.10** in Rust. The
player is shown a field of randomly placed squares whose colors are near-identical
variations of one another, and must click/tap the one square that carries the exact
target color. It runs natively and in the browser via WebAssembly (a prebuilt WASM
bundle is committed under `docs/` and served by GitHub Pages).

The crate is still named `bevy-tetris` in `Cargo.toml` (leftover from the template
this project started from) — the game itself has nothing to do with Tetris. Don't
"fix" this name casually; the WASM artifacts in `docs/` are named `puzzle_wasm.*`,
so a rename touches the web build too.

In-game text is **Brazilian Portuguese** ("Jogar", "Voltar", "Fim de Jogo",
"Ver Historico"). Keep new user-facing strings in Portuguese; code, identifiers and
comments are in English.

## Commands

```bash
cargo run            # run natively (debug; deps are built at opt-level 3, our code at 1)
cargo build          # build
cargo check          # fast type-check — prefer this for verifying edits
cargo build --release
cargo test           # runs, but there are currently no tests in the repo
```

WASM: `Cargo.toml` sets `runner = "wasm-server-runner"` for
`wasm32-unknown-unknown`, so the intended local web loop is

```bash
cargo run --target wasm32-unknown-unknown
```

The committed `docs/` bundle (`puzzle_wasm.js`, `puzzle_wasm_bg.wasm`, `index.html`,
`assets/digital7mono.ttf`) is generated with `wasm-bindgen` using the output name
`puzzle_wasm` and is what GitHub Pages serves. `docs/index.html` mounts the game on
`<canvas id="canvas">`, which matches `canvas: Some("#canvas".into())` in
`src/main.rs`. Regenerating the bundle means re-running wasm-bindgen and committing
the new binaries; the build script for it is not in the repo.

CI (`.github/workflows/rust.yml`) runs `cargo build --verbose` and
`cargo test --verbose` on `macos-latest`, triggered on push/PR to `main`. Note the
default branch here is **`master`**, so CI does not currently fire for normal work.

Native Linux builds need Bevy's system deps (`libasound2-dev`, `libudev-dev`,
`pkg-config`, `build-essential`) — see `.devcontainer/setup.sh`. Without them the
build dies in the `alsa-sys` build script with "The system library `alsa` ... was not
found", which is an environment problem, not a code problem. Sandboxes that can't
install system packages therefore cannot type-check this crate at all; say so rather
than reporting a change as verified.

## Architecture

Everything is a binary crate rooted at `src/main.rs`; there is no `lib.rs`. Modules
are declared in `main.rs` and glob-imported (`use game::*;` etc.), which means
nested modules are reachable from the crate root — you will see both
`crate::game::ui::...` and the shorter `crate::ui::...` used for the same path. Both
compile; prefer the explicit `crate::game::ui::...` form in new code.

### App wiring (`src/main.rs`)

Registers `AppState`, the two global events, and four plugins: `MainMenuPlugin`,
`GamePlugin`, `WasmPlugin`, `InteractionAnimationPlugin`. `DefaultPlugins` is added
*after* the custom plugins along with `ShapePlugin` (bevy_prototype_lyon). Window
config lives here: `PresentMode::AutoNoVsync`, `fit_canvas_to_parent: true`, resize
constraints tuned for mobile portrait (min 320x480).

### States

`AppState` (in `src/main.rs`) drives the whole app:

| State | Meaning |
| --- | --- |
| `MainMenu` (default) | Game-mode selection screen |
| `Game` | Active puzzle round |
| `Paused` | Declared but currently unused |
| `History` | Paginated list of played levels |
| `LevelHistory` | Replay/inspection of one past level |
| `GameOverResume` | End-of-run stats screen |
| `GameOver` | "Fim de Jogo" press-to-continue screen |

`SimulationState` in `src/game/mod.rs` is declared but not registered with the app.

**State transitions go through an event, not directly.** UI systems send
`TransitionToStateEvent { state }`; `src/wasm/systems.rs::transition_to_state` reads
it and calls `NextState::set`. This indirection exists because setting `NextState`
straight from a UI interaction system behaved badly in the web build. Follow this
pattern for new UI-driven transitions. (`src/game/ui/hud/systems/interactions.rs`
still sets `NextState` directly — that's the older style.)

Debug keyboard shortcuts in `src/systems.rs`: `G` → Game, `M` → MainMenu,
`H` → GameOver, `Esc` → quit.

### Module layout

```
src/
  main.rs                     App setup, AppState, PIXELS_PER_METER/RESOLUTION consts
  systems.rs                  spawn_camera, BackgroundTranstion component, debug hotkeys, exit_game
  events.rs                   GameOver, TransitionToStateEvent, InteractionAnimationEvent
  pagination.rs               Pagination resource (page math + the container Entity)
  interaction_animation.rs    Expanding white square spawned where the player clicked
  wasm/                       WasmPlugin — consumes TransitionToStateEvent
  main_menu/                  MainMenuPlugin — one Play button per GameMode
  game/
    mod.rs                    GamePlugin — adds GameUIPlugin + PuzzlePlugin
    puzzle/                   Core gameplay (see below)
    ui/
      hud/                    Pause ("||") button, history back button
      game_history_menu/      Paginated level list
      game_over_menu/         Game-over + stats screens
    score/                    ORPHANED — see "Dead code" below
assets/                       digital7mono.ttf (the only asset actually loaded), buttons.png
docs/                         Prebuilt WASM bundle for GitHub Pages
```

Each UI feature follows the same four-part convention — copy it for new UI:

```
feature/
  mod.rs        the Plugin impl; registers systems against OnEnter/OnExit/OnUpdate
  components.rs marker Components (one per interactive element)
  styles.rs     const Style/Color values + get_*_text_style(asset_server) helpers
  systems/
    mod.rs          pub mod declarations
    layout.rs       spawn_* / despawn_* / build_* entity trees
    interactions.rs Query<&Interaction, (Changed<Interaction>, With<Marker>)> handlers
    updates.rs      (game_history_menu only) rebuilds contents on demand
```

Spawn on `OnEnter(state)`, despawn on `OnExit(state)`, gate per-frame systems with
`run_if(in_state(...))` or `.in_set(OnUpdate(...))`.

### Gameplay core (`src/game/puzzle/`)

`components.rs` holds the data model, `systems.rs` the behavior.

- **`ColorPuzzle`** (Resource, `Reflect`) — the puzzle state machine: score,
  `current_colors`, `correct_color_index`, difficulty knobs, window dimensions.
  `generate_colors()` picks a base color with one predominant channel, then derives
  N near-duplicates by adding ≤0.1 per channel; the "correct" one is the unmodified
  base. Count comes from `get_score_color_count()` =
  `difficulty * score_to_increase_difficulty_formula(score) * objects_per_difficulty`,
  so the field gets denser as the score climbs.
- **`GameMode`** — `Infinite` (no timer), `AgainstTheClock` (60s), `TimeTrial`
  (30s, +3s per correct pick). `setup(&mode)` reconfigures `ColorPuzzle`;
  `GameMode::as_str()` returns the Portuguese label shown in the menu.
- **`GameTimer`** (Resource) — a single `Timer`; starts paused. When it finishes,
  `render_remaining_time` transitions to `GameOverResume`.
- **`GameHistory`** (Resource) — every `LevelHistory` (clicked position, all
  `LevelColor`s, which was correct, whether it scored) plus aggregate stats
  (`levels_played`, `total_score`, `max_streak`, `total_time`). Drives both the
  history menu and the level replay.
- **`PuzzleColor`** (Component) — one per rendered square. `PuzzleColorGame` is the
  broad marker used by `despaw_objects` (sic) to clear the board on `OnExit(Game)`.

Round flow: `start_puzzle_level` (OnEnter Game) sizes the puzzle to the window and
fires `StartLevelEvent` → `spawn_objects` despawns the old board and spawns the new
squares at non-overlapping random positions (rejection sampling, 100 tries max) →
`player_interaction` hit-tests mouse release / touch against the squares, sends
`InteractionAnimationEvent` + `LastInteractionEvent` and another `StartLevelEvent` →
`store_last_interaction_state` appends to `GameHistory`.

Rendering uses **bevy_prototype_lyon** `ShapeBundle` + `Fill`, not sprites. Squares
use `RectangleOrigin::BottomLeft`, so a square's `Transform` is its bottom-left
corner — `mouse_hover`/`cord_is_intersecting` in `systems.rs` depend on this. Depth
is handled by incrementing `z` by 0.1 per square.

The background is the camera clear color. `BackgroundTranstion` (`src/systems.rs`,
spawned on the camera entity) lerps from the previous round's target color to the
current one over `transition_seconds`; `background_transition` applies it each
frame. `player_interaction` **ignores input while a transition is running** — if new
input handling seems dead, check `is_in_transition()` first.

## Conventions and gotchas

- Bevy **0.10** API specifically: `add_system(x.in_schedule(OnEnter(S)))`,
  `.in_set(OnUpdate(S))`, `Interaction::Clicked` (not `Pressed`), `Style::size` with
  `Size::new(...)`, `app_state.0` to read the current state. Do not port code from
  0.11+ examples without translating it.
- Event readers overwhelmingly use `events.iter().next()` and process only the first
  event per frame. That is the established pattern here; it also means bursts of
  events get dropped. Preserve it unless a bug specifically requires draining.
- Names contain typos that are load-bearing (referenced across files): `despaw_objects`,
  `BackgroundTranstion`, `HistoryButtom`, `HistoryBackButtom`, `last_interraction_event_writer`.
  Rename only deliberately and repo-wide.
- Fonts are loaded ad hoc with `asset_server.load("digital7mono.ttf")` at each use
  site; `assets/buttons.png` is unreferenced.
- `Cargo.lock` is gitignored, so dependency versions float within their semver
  ranges — an unexpected build break may be an upstream release, not your change.
- `bevy_rapier2d`, `bevy-inspector-egui`, `lazy_static` and `wasm-bindgen` are
  declared in `Cargo.toml` but unused in `src/`. They still cost build time.
- No test suite, no `rustfmt.toml`/`clippy.toml`, and formatting in the tree is
  inconsistent (mixed indentation, trailing whitespace). Running `cargo fmt` across
  the repo would produce a huge unrelated diff — format only what you touch.

### Dead code to be aware of

- `src/game/score/` (`ScorePlugin`, `Score`, `HighScores`) is **not declared** in
  `src/game/mod.rs` and is never compiled. `events::GameOver` is only used by it.
  Treat it as inert; if score persistence is wanted, wiring this up is the starting
  point.
- `src/game/systems.rs` is empty but declared (`mod systems;`).
- `AppState::Paused` and `SimulationState` are declared and unused.

## Git workflow

Default branch is `master`. Feature work happens on `claude/*` branches; push with
`git push -u origin <branch>`. Don't open a PR unless asked. Commit messages in the
existing history are terse ("new", "fix"); write clearer ones than that.
