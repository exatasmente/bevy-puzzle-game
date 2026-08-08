# CLAUDE.md

Guidance for AI assistants working in this repository.

## What this project is

A color-discrimination puzzle game ("Color Puzzle") built with **Bevy 0.10** in Rust.
The player is shown a grid of squares that all share one color except a single odd
one, and must tap the odd one. It runs natively and in the browser via WebAssembly (a
prebuilt WASM bundle is committed under `docs/` and served by GitHub Pages).

**The board never hides the answer in the background.** An earlier version painted the
window with the target color, so the correct square was invisible and the task was
really "spot the gap in the layout" — a pop-out the visual system solves in
milliseconds, which made the color-distance difficulty dial almost irrelevant. The
background is now the app's dark ground with a hint of the round's hue, and the round
is decided by comparing squares to each other. Do not reintroduce a background that
matches any square.

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

WASM: `.cargo/config.toml` sets `runner = "wasm-server-runner"` for
`wasm32-unknown-unknown`, so the local web loop is

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
found". That is an environment problem, not a code problem.

**Checking without those packages.** In a sandbox that cannot install them, you can
still type-check, because `cargo check` never links. Write stub `alsa.pc` and
`libudev.pc` files into a directory and point pkg-config at it:

```bash
# minimal .pc: prefix/libdir/includedir + Name/Description/Version/Libs/Cflags
PKG_CONFIG_PATH=/path/to/stubs cargo check
```

To get as far as a linked binary, the runtime libraries usually exist even when the
`-dev` packages don't; symlink `libasound.so.2` → `libasound.so` and `libudev.so.1` →
`libudev.so` into a directory and add `RUSTFLAGS="-L /that/dir"` to `cargo build`.

**Linking is not running.** Bevy still needs a GPU adapter at startup and panics with
"Unable to find a GPU!" without one. `xvfb-run` alone is not enough: wgpu needs either
a Vulkan ICD (`mesa-vulkan-drivers`, i.e. lavapipe) or `libEGL` for the GL backend, and
a bare container typically has neither — Mesa's `swrast_dri.so` and `libGL` are not
sufficient, because wgpu's GL backend goes through EGL. If you cannot install those,
say plainly that the change was type-checked but never executed. Most Bevy mistakes
(missing resources, `single()` on an empty query, system-ordering races) compile fine
and only fail at runtime, so a clean `cargo check` is weaker evidence than it looks.

## Architecture

Everything is a binary crate rooted at `src/main.rs`; there is no `lib.rs`. Modules
are declared in `main.rs` and glob-imported (`use game::*;` etc.), which means
nested modules are reachable from the crate root — you will see both
`crate::game::ui::...` and the shorter `crate::ui::...` used for the same path. Both
compile; prefer the explicit `crate::game::ui::...` form in new code.

### App wiring (`src/main.rs`)

Registers `AppState`, the two global events, and five plugins: `MainMenuPlugin`,
`GamePlugin`, `WasmPlugin`, `FeedbackPlugin`, `InteractionAnimationPlugin`.
`DefaultPlugins` is added
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

**State transitions go through an event, not directly.** UI systems send
`TransitionToStateEvent { state }`; `src/wasm/systems.rs::transition_to_state` reads
it and calls `NextState::set`. This indirection exists because setting `NextState`
straight from a UI interaction system behaved badly in the web build. Every UI
interaction system now follows it; the only places that set `NextState` directly are
non-UI systems (`tick_game_timer`, `handle_new_game_event`, the debug hotkeys).

Debug keyboard shortcuts in `src/systems.rs`: `G` → Game, `M` → MainMenu,
`H` → GameOver, `Esc` → quit.

### Module layout

```
src/
  main.rs                     App setup, AppState, PIXELS_PER_METER/RESOLUTION consts
  theme.rs                    Design tokens: palette, type scale, spacing, button states
  oklab.rs                    Perceptual color space; the unit the difficulty is set in
  layout.rs                   RelayoutEvent: screens rebuild when the window resizes
  feedback.rs                 FeedbackPlugin — pop, floating text, screen shake, banners, reveal
  storage.rs                  localStorage on wasm, no-op on native
  systems.rs                  spawn_camera, BackgroundTranstion component, debug hotkeys, exit_game
  events.rs                   TransitionToStateEvent, InteractionAnimationEvent
  pagination.rs               Pagination resource (page math + the container Entity)
  interaction_animation.rs    Per-pick world feedback: success ring, miss cross, answer reveal
  wasm/                       WasmPlugin — consumes TransitionToStateEvent
  main_menu/                  MainMenuPlugin — one mode card per GameMode, with best score
  game/
    mod.rs                    GamePlugin — adds GameUIPlugin + ScorePlugin + PuzzlePlugin
    puzzle/                   Core gameplay (see below)
    score/                    ScorePlugin — persisted per-mode BestScores, LastRunOutcome
    ui/
      hud/                    Top bar: score, streak, timer, level bar, pause
      game_history_menu/      Pause screen: paginated round review, continue, end run
      game_over_menu/         End-of-run summary + "fim de jogo" interstitial
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
    updates.rs      per-frame value updates (hud) or on-demand rebuilds (game_history_menu)
```

Spawn on `OnEnter(state)`, despawn on `OnExit(state)`, gate per-frame systems with
`run_if(in_state(...))` or `.in_set(OnUpdate(...))`.

**Every screen that sizes itself from the window must handle `RelayoutEvent`**
(`src/layout.rs`). Menus bake pixel widths — and the fitted font size of every label —
into their node trees, so a window that changes size after the screen was built leaves
it wrong. This is the normal case on the web, where `fit_canvas_to_parent` resizes the
canvas a frame or two after the main menu has already been built at Bevy's default
1280x720. The handlers despawn and rebuild; the pause screen just re-sends its own
`SpawnPaginationEvent`.

### Gameplay core (`src/game/puzzle/`)

`components.rs` holds the data model, `systems.rs` the behavior.

- **`ColorPuzzle`** (Resource, `Reflect`) — the puzzle state machine: score,
  `current_colors`, `base_color`, `correct_color_index`, window dimensions.
  `background_color()` is the app ground mixed 16% toward the round's base — never any
  square's color. `is_correct_color(index)` compares *indices*, because every square
  but one now shares a color by design.
- **Difficulty** is a 1-based level derived from the score via the `LEVEL_START_SCORES`
  table (`level_for_score`, `score_for_level`, `progress_to_next_level`). Three dials
  move with it: `color_count_for_level` (4 → 12 squares), `color_delta_for_level`
  (0.080 → 0.018) and, in `Memory`, `preview_seconds_for_level` (1.7s → 0.7s).
- **Color generation** lives in `generate_colors()` and is written in **Oklab**
  (`src/oklab.rs`), not sRGB. A round is one base color plus one odd color exactly
  `color_delta_for_level` away from it in a random, mostly-chromatic direction. The
  unit matters: 0.05 on an sRGB channel is glaring in a grey and invisible in a dark
  blue, so the old per-channel variation made a level mean something different every
  round. `oklab::to_color` returns `None` outside the sRGB gamut and the generator
  retries rather than clamping — a clamped color would be closer to the base than the
  level claims, and so easier.
- **`GameMode`** — `Infinite` (no timer), `AgainstTheClock` (60s), `TimeTrial`
  (30s, +3s per correct pick), `Memory` (no timer; the board is shown, then blanks).
  `setup(&mode)` reconfigures `ColorPuzzle`; `as_str()`/`description()` return the
  Portuguese labels, `accent()` the mode's identity color, `storage_key()` the stable
  key for persisted bests, `is_timed()` and `hides_colors()` the behavior switches.
  Descriptions have about 25 characters before the type shrinks past reading size —
  shorten the string rather than the floor.
- **`MemoryPhase`** (Resource) — drives a `Memory` round: preview, then hidden.
  `hide_memory_board` repaints every square's `Fill` when the preview ends, which
  leaves the entities alone so the hit test and the answer reveal keep working.
  `player_interaction` refuses input while previewing, and repaints the true colors
  on a miss so a missed round still teaches.
- **`GameTimer`** (Resource) — a single `Timer`; starts paused. When it finishes,
  `tick_game_timer` transitions to `GameOverResume`. Infinite runs never expire, so
  they end only via "ENCERRAR PARTIDA" on the pause screen.
- **`BestScores`** (Resource, `src/game/score/`) — per-mode personal bests, loaded at
  startup and saved on each run end. `LastRunOutcome` carries score/best/is_record to
  the game-over screen; the screen orders itself `.after(RecordOutcomeSet)` so it
  never renders the previous run's numbers.
- **`GameHistory`** (Resource) — every `LevelHistory` (clicked position, all
  `LevelColor`s, which was correct, whether it scored) plus aggregate stats
  (`levels_played`, `total_score`, `max_streak`, `total_time`). Drives both the
  history menu and the level replay.
- **`PuzzleColor`** (Component) — one per rendered square, carrying its position and
  its `size`. Size is stored per square, not read from `ColorPuzzle::shape_size`, so a
  replayed round is drawn and hit-tested at the size it was played at.
  `PuzzleColorGame` is the broad marker used by `despaw_objects` (sic) to clear the
  board on `OnExit(Game)`.
- **`BoardGrid`** — the board layout. `ColorPuzzle::grid_for_count` picks the column
  count that yields the largest square cell, centers the grid in the window minus
  `HUD_RESERVED_HEIGHT`, and centers a short last row. Squares are never placed at
  random: the game is a color comparison, and a grid is what lets the eye do it.

Round flow: `start_puzzle_level` (OnEnter Game) sizes the puzzle to the window and
fires `StartLevelEvent` → `spawn_objects` despawns the old board, lays the new squares
out on the `BoardGrid` for their count and writes the cell size back to
`ColorPuzzle::shape_size` →
`player_interaction` hit-tests mouse release / touch against the squares, sends
`InteractionAnimationEvent` + `LastInteractionEvent` and another `StartLevelEvent` →
`store_last_interaction_state` appends to `GameHistory`.

`player_interaction` skips the frame entirely when any UI element reports a non-`None`
`Interaction`; without that, tapping a HUD button also counts as a pick on the board
and breaks the player's streak.

### Feedback layer

`src/feedback.rs` holds the reusable primitives — `PopAnim` (scale punch, works on UI
nodes because Bevy's layout owns only `Transform::translation`), `spawn_floating_text`,
`ScreenShakeEvent` / `ScreenShake` (on the camera, next to `BackgroundTranstion`),
`BannerEvent` for level ups and streak milestones, and `RevealIn` for staggered text.
`src/interaction_animation.rs` consumes `InteractionAnimationEvent` and renders the
per-pick response: green ring and "+1" on a hit, red cross plus shake and a blinking
outline over the correct square on a miss.

The event carries `scored`, `bonus_seconds` and `correct_position` precisely so hit and
miss can look different — if you add a new outcome, extend the event rather than
inferring the result downstream.

Rendering uses **bevy_prototype_lyon** `ShapeBundle` + `Fill`, not sprites. Squares
use `RectangleOrigin::BottomLeft`, so a square's `Transform` is its bottom-left
corner — `mouse_hover` in `systems.rs` depends on this. Depth is handled by
incrementing `z` by 0.1 per square.

**Pointer coordinates.** Bevy 0.10's `viewport_to_world_2d` maps its argument straight
to NDC without flipping y, so it wants a bottom-left origin — which is what
`Window::cursor_position` gives. Touches are the exception: `bevy_winit` passes them
through in winit's top-left convention, so `player_interaction` flips those before
converting. Getting this wrong mirrors every pick vertically.

The background is the camera clear color. `BackgroundTranstion` (`src/systems.rs`,
spawned on the camera entity) lerps from the previous round's background to the
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
- All colors, font sizes and spacing come from `src/theme.rs`. Feature `styles.rs`
  files re-export from it and keep only their own layout `Style` consts — do not
  reintroduce per-screen palettes.
- **Build every label with `theme::wrapped_text`, and size cards and buttons in
  pixels** (`theme::content_width` from the window width, `theme::button_style`).
  `digital7mono.ttf` draws well past the vertical metrics it reports, so wrapped lines
  and merely-adjacent nodes overlap into a smear. `wrapped_text` therefore keeps each
  label on one line — shrinking the type via `GLYPH_ADVANCE_RATIO` until it fits —
  and adds vertical margin. Percentage-width buttons cannot be fitted against and will
  overflow again.
- **User-facing strings must be unaccented ASCII.** `digital7mono.ttf` is a
  seven-segment display font with a narrow glyph set: "ç", "ó" and symbols like "✓"
  render as blanks. Write "HISTORICO", "SEQUENCIA", "OK"/"X".
- Interactive elements are at least `theme::TOUCH_TARGET` (48px) in both axes; the
  game is played on phones.
- `Cargo.lock` is gitignored, so dependency versions float within their semver
  ranges — an unexpected build break may be an upstream release, not your change.
- `bevy_rapier2d`, `bevy-inspector-egui`, `lazy_static` and `wasm-bindgen` are
  declared in `Cargo.toml` but unused in `src/`. They still cost build time.
  `web-sys` is declared under a `cfg(target_arch = "wasm32")` target table and is
  used only by `src/storage.rs`.
- No test suite, no `rustfmt.toml`/`clippy.toml`, and formatting in the tree is
  inconsistent (mixed indentation, trailing whitespace). Running `cargo fmt` across
  the repo would produce a huge unrelated diff — format only what you touch.

### Visual language

The palette in `theme.rs` is the mock-up's: near-black violet ground (`BACKGROUND`),
panels a step above it (`SURFACE`, `SURFACE_RAISED`), and saturated accents —
`PRIMARY` purple for the one action a screen wants taken, `DANGER` crimson for the one
that ends a run, `SUCCESS`, `ACCENT`, `LIME`, `INFO`. Each `GameMode` owns one of them
via `accent()`.

Bevy 0.10's UI cannot stroke a node, so a border is a wrapper node painted in the
border color with `HAIRLINE` padding, containing the real panel — see
`theme::outlined_style`. Note that Taffy sizes the *content* box: a node at
`Percent(100)` with padding is wider than its parent, and a `Px` node's padding is
added outside its width. Both mistakes push whole screens off the edge, and both were
in this codebase.

The board squares are deliberately flat — no stroke, no glow, unlike the mock-up's
blocks. An outline in the square's own color would make the odd square identifiable
from its edge rather than its fill, which is the entire puzzle.

### Dead code to be aware of

- `AppState::Paused` is declared and unused — pausing goes to `History`, which doubles
  as the pause screen.
- `PuzzleColor::index` is stored but never read.
- `PIXELS_PER_METER` and `RESOLUTION` in `main.rs` are leftovers from the template.

## Git workflow

Default branch is `master`. Feature work happens on `claude/*` branches; push with
`git push -u origin <branch>`. Don't open a PR unless asked. Commit messages in the
existing history are terse ("new", "fix"); write clearer ones than that.
