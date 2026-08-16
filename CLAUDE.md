# CLAUDE.md
Guidance for AI assistants working in this repository.
 
## What this project is

A color-discrimination puzzle game ("Color Puzzle") built with **Bevy 0.19** in Rust.
The board is a honeycomb of **congruent regular hexagons**. Some cells are empty and
simply show the ground; the filled ones are grouped into blobs of colour, and the
board reads as a mosaic. One filled cell wears its group's colour moved by the level's
delta — it is the only cell on the board wearing exactly that colour, and it is the
answer. It runs natively and in the browser via WebAssembly, which GitHub Pages builds
from source on every push to `master` — see the Commands section, because the committed
`docs/` bundle is *not* what gets served.

**The ground sweeps.** At the start of every round the background travels through all
of the round's colours over one second and stops on the answer's. Each group vanishes
for an instant as the ground passes its colour; the answer is the one that vanishes at
the end and *stays* gone. That gives the round two ways to be solved, and they suit
different players: spot the cell whose colour does not match the blob it sits in, or
watch the sweep and remember which hole arrived last. Do not shorten the sweep or make
it instant — it is the round's second channel of information, not a transition effect.

**The empty cells are deliberate, and they used to be a bug.** Under the old Voronoi
cut a dropped cell left a second background-coloured hole indistinguishable from the
answer, and the layout worked hard to avoid one. Now holes are the point: what
separates the answer from them is what the sweep showed. This is also why the lattice
could go regular after a long stretch of insisting it must not — the answer is no
longer "the cell that is missing", so a known address gives nothing away.

`Mosaic` is the odd mode out: its puzzle is a pattern rather than a colour, so it still
uses a plain grid, a static ground and no sweep.

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
cargo test           # generator invariants (board, mosaic_pattern, wfc, the difficulty curve)
```

WASM: `.cargo/config.toml` sets `runner = "wasm-server-runner"` for
`wasm32-unknown-unknown`, so the local web loop is

```bash
cargo run --target wasm32-unknown-unknown
```

**GitHub Pages does not serve the committed `docs/` bundle.**
`.github/workflows/main.yml` ("Build and Deploy WebAssembly") fires on every push to
`master`, builds `--release --target wasm32-unknown-unknown` itself, runs `wasm-bindgen`
(pinned to 0.2.126), then copies **`docs/index.html`** and the whole **`assets/`** tree
next to it and publishes that. So what is deployed comes from the *source*, and the only
committed file it actually uses is `index.html` — which is why the AudioContext shim
lives there and matters. Verified by fetching the site: its `puzzle_wasm_bg.wasm` is
29 MB against the 21 MB committed under `docs/`, a different build of the same code.

`docs/puzzle_wasm*.{js,wasm}` are therefore dead weight — 21 MB of binaries nothing
serves. Do not bother regenerating them to "update the site"; push to `master` instead.
`docs/index.html` mounts the game on `<canvas id="canvas">`, which matches
`canvas: Some("#canvas".into())` in `src/main.rs`.

CI (`.github/workflows/rust.yml`) runs `cargo build --verbose` and
`cargo test --verbose` on `macos-latest`, triggered on push/PR to `main`. Note the
default branch here is **`master`**, so CI does not currently fire for normal work.

Native Linux builds need Bevy's system deps (`libasound2-dev`, `libudev-dev`,
`libwayland-dev`, `libxkbcommon-dev`, `pkg-config`, `build-essential`) — see
`.devcontainer/setup.sh`. Without them the build dies in a `*-sys` build script with
"The system library `alsa` ... was not found" or the equivalent for wayland. That is
an environment problem, not a code problem. The two wayland/xkb packages became
required at 0.19, through `accesskit_winit → winit`, and are needed **regardless of
which windowing feature is on** — an X11-only build wants them too.

Also note the toolchain floor: **rustc 1.95**, for bevy 0.19.1.

**Checking without those packages.** In a sandbox that cannot install them, you can
still type-check, because `cargo check` never links. Write stub `.pc` files into a
directory and point pkg-config at it. Seven are needed now: `alsa`, `libudev`,
`wayland-client`, `wayland-cursor`, `wayland-egl`, `xkbcommon` and `libdecor-0`.

**The `Libs:` line must name the real library, not the package.** `alsa.pc` links
`-lasound` and `libudev.pc` links `-ludev`; a stub that echoes the package name
type-checks fine and then fails at link with "unable to find library -lalsa", which
looks like a missing package rather than a bad stub. Measured the hard way.

```bash
write_pc() {  # write_pc <pkg-name> <library-name>
  printf 'prefix=/usr\nexec_prefix=${prefix}\nlibdir=${prefix}/lib\nincludedir=${prefix}/include\n\nName: %s\nDescription: stub\nVersion: 999.0\nLibs: -l%s\nCflags:\n' "$1" "$2" > stubs/$1.pc
}
write_pc alsa asound;                 write_pc libudev udev
write_pc wayland-client wayland-client; write_pc wayland-cursor wayland-cursor
write_pc wayland-egl wayland-egl;     write_pc xkbcommon xkbcommon
write_pc libdecor-0 decor-0
PKG_CONFIG_PATH=$PWD/stubs cargo check
```

`cargo check --target wasm32-unknown-unknown` needs none of this — it is worth running
on its own, because it is the build that actually ships and `uuid`/`getrandom` only
break there.

**To get as far as a linked binary** — which `cargo test` needs, since it builds and
runs a binary — the runtime libraries usually exist even when the `-dev` packages
don't, but only under their versioned sonames. Symlink each to the bare `.so` name the
linker asks for and add that directory to `RUSTFLAGS`. Only three are actually linked
(alsa, udev, wayland-client); the rest are dlopened:

```bash
for f in libasound.so.2 libudev.so.1 libwayland-client.so.0 \
         libwayland-cursor.so.0 libwayland-egl.so.1 libxkbcommon.so.0; do
  ln -sf /usr/lib/x86_64-linux-gnu/$f linklibs/${f%.so.*}.so
done
PKG_CONFIG_PATH=$PWD/stubs RUSTFLAGS="-L $PWD/linklibs" cargo test
```

Note that changing `RUSTFLAGS` invalidates the cache and rebuilds all of Bevy, so set
it on the first run rather than discovering you need it after a clean `cargo check`.

**Linking is not running.** The *native* binary needs a GPU adapter at startup and
panics with "Unable to find a GPU!" without one. `xvfb-run` alone is not enough: wgpu
needs either a Vulkan ICD (`mesa-vulkan-drivers`, i.e. lavapipe) or `libEGL` for the
GL backend, and a bare container typically has neither — Mesa's `swrast_dri.so` and
`libGL` are not sufficient, because wgpu's GL backend goes through EGL.

**But that is only the native build, and it is not a reason to stop at type-checking.**
The wasm build needs none of it: the browser supplies WebGL itself, through SwiftShader
when headless, and a container with Chromium can therefore *play the game*. Do this
before claiming anything works — most Bevy mistakes (missing resources, `single()` on
an empty query, plugin ordering) compile fine and only fail at runtime, so a clean
`cargo check` is weaker evidence than it looks. The 0.19 port shipped a `main.rs` that
aborted at startup and passed every type-check and all 33 tests on the way there.

```bash
cargo build --release --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.126 --locked
wasm-bindgen --target web --out-dir dist --out-name puzzle_wasm \
  target/wasm32-unknown-unknown/release/bevy-tetris.wasm
cp docs/index.html dist/ && cp -r assets dist/assets       # same as the workflow
(cd dist && python3 -m http.server 8080 --bind 127.0.0.1 &)
```

Then drive it with Playwright against the bundled Chromium, launched with
`--enable-unsafe-swiftshader --use-gl=angle --use-angle=swiftshader`. Give it ~45s to
boot: the local bundle carries debug info and is ~115 MB, and software rendering is
slow. Listen to `pageerror` — a Rust panic arrives there as `RuntimeError: unreachable`.

**A panic before `LogPlugin` is installed has no message at all.** `console_error_panic_hook`
is set up by Bevy during plugin build, so anything that aborts earlier gives a bare
`unreachable` and an empty console — indistinguishable, from outside, from a hang. The
stack trace in `pageerror` still names the frame (`...::init_state::<AppState>`), and
that is usually enough to find it. A canvas still at 300x150, the HTML default, means
Bevy never took it over and the app never started.

## Architecture

Everything is a binary crate rooted at `src/main.rs`; there is no `lib.rs`. Modules
are declared in `main.rs` and glob-imported (`use game::*;` etc.), which means
nested modules are reachable from the crate root — you will see both
`crate::game::ui::...` and the shorter `crate::ui::...` used for the same path. Both
compile; prefer the explicit `crate::game::ui::...` form in new code.

### App wiring (`src/main.rs`)

Registers `AppState`, the two global events, and six plugins: `MainMenuPlugin`,
`GamePlugin`, `WasmPlugin`, `FeedbackPlugin`, `GameAudioPlugin`,
`InteractionAnimationPlugin`.
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
  board.rs                    Regular pointy-top hex lattice, uniform gaps (has tests)
  mosaic_pattern.rs           Which cells are empty and which colour blob each joins (has tests)
  oklab.rs                    Perceptual color space; the unit the difficulty is set in
  audio.rs                    GameAudioPlugin — pick/level sounds, music, Volume
  wfc.rs                      Wave function collapse; generates the Mosaic board (has tests)
  layout.rs                   RelayoutEvent: screens rebuild when the window resizes
  feedback.rs                 FeedbackPlugin — pop, floating text, screen shake, banners, reveal
  storage.rs                  localStorage on wasm, no-op on native
  systems.rs                  spawn_camera, BackgroundTranstion (the sweep), debug hotkeys, exit_game
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
      hud/                    Top bar: score, streak, timer, pause; level bar and lives
      game_history_menu/      Pause screen: paginated round review, continue, end run
      game_over_menu/         End-of-run summary + "fim de jogo" interstitial
assets/                       digital7mono.ttf, sfx/*.wav, buttons.png (unused)
docs/                         Prebuilt WASM bundle for GitHub Pages
tools/make_sounds.py          Regenerates assets/sfx/ — pure stdlib, no dependencies
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
  `current_colors`, `current_slots`, `base_color`, `correct_color_index`, window
  dimensions. `background_color()` returns the answer's own color, which is what makes
  it invisible; in `Mosaic` it returns a dimmed ground instead, because that mode's
  pieces all have to be seen. `is_correct_color(index)` compares *indices*.
- **Colour is a small palette, not a colour per piece.** `palette_size_for_level` gives
  `K` (4 → 8) groups, and every filled cell wears its group's colour exactly. `K` is
  small on purpose: the sweep visits *colours*, and at 16 columns the board holds ~530
  cells — 530 stops in one second is 2ms each and shows nothing, while `K` stops make a
  whole blob blink at a time, which is what the eye can actually read.
- **The answer is the only cell alone in its colour.** If it shared a colour with its
  group, the ground settling would erase the whole group and the round would have
  several defensible answers — the same fairness bug the `Mosaic` tests once caught.
  `answer_color` therefore keeps it clear of every *other* group by `(delta*2).max(0.03)`
  while sitting exactly `delta` from its own.
- **Difficulty is a formula, and it never ends.** `score_for_level(L) = 2 + 3L(L−1)/2`
  reproduces the nine-entry table it replaced (there is a test); `level_for_score`
  inverts it in closed form. `score_for_level` works in `u64` with `saturating_*`
  because `usize` is 32 bits on wasm32 and the direct form overflows near level 53 500.
  Four dials move with the level, all monotone, none with a ceiling in the level:
  `columns_for_level` (4 → 16), `empty_share_for_level` (.20 → .50),
  `palette_size_for_level` (4 → 8) and `color_delta_for_level` (0.050 → `MIN_COLOR_DELTA`
  = 0.010). Below about 0.008 in Oklab the discrimination is a coin flip, so the
  difficulty *plateaus* rather than rising forever — the levels keep counting, the
  challenge tends to a limit. In `Memory`, `preview_seconds_for_level` (1.7s → 0.7s)
  moves too.
- **Colour generation** lives in `generate_colors()` and is written in **Oklab**
  (`src/oklab.rs`), not sRGB. The unit matters: 0.05 on an sRGB channel is glaring in a
  grey and invisible in a dark blue, so a level would mean something different every
  round. `oklab::to_color` returns `None` outside the sRGB gamut and the generator
  retries rather than clamping — a clamped colour would be closer to the base than the
  level claims, and so easier. **`palette()` walks an arc of hue rather than nudging in
  random directions**: the separation the round needs is a large distance in Oklab,
  random candidates that far out mostly fall outside the gamut, the retry budget runs
  out and every group lands on the fallback. That produced a board in one flat blue.
  An arc is separated by construction. `every_colour_group_is_visibly_its_own` pins it.
- **`GameMode`** — `Infinite` (no timer), `AgainstTheClock` (60s), `TimeTrial`
  (30s, +3s per correct pick), `Memory` (no timer; the board is shown, then blanks).
  `setup(&mode)` reconfigures `ColorPuzzle`; `as_str()`/`description()` return the
  Portuguese labels, `accent()` the mode's identity color, `storage_key()` the stable
  key for persisted bests, `is_timed()` and `hides_colors()` the behavior switches.
  Descriptions have about 25 characters before the type shrinks past reading size —
  shorten the string rather than the floor.
- **Every mode charges a miss, and exactly one way.** Missing used to be free
  everywhere — nothing subtracted, and `tick_game_timer` even pauses the clock for the
  length of the hold — so guessing beat looking. The untimed modes now run on
  **lives** (`GameMode::starting_lives`, three), which is also the only thing that can
  end a run in `Infinite`/`Memory`/`Mosaic`; the timed ones charge
  `GameMode::miss_penalty_seconds` instead (3s / 2s), by winding the timer's *elapsed*
  forward rather than shortening its duration, so `TimeTrial`'s bonus seconds keep
  meaning what they meant. Never both in one mode: two falling numbers is one more
  than the player can watch while reading a field of near-identical colours.
  `every_mode_has_one_way_to_run_out` pins the split.
  Lives live on `ColorPuzzle` rather than in a resource of their own, because
  `setup(&mode)` is the single funnel all three run starts go through (the menu's play
  button, "jogar novamente", a resumed run) — a separate resource would have to be
  reset at each of them. A life comes back every `LEVELS_PER_EXTRA_LIFE` (5) levels,
  capped, and rides on the level-up banner ("NIVEL 5  +1 VIDA") rather than sending a
  second one, since `handle_banner_events` keeps only the newest. Losing one is
  reported by the HUD pips alone — a centred banner would cover the answer reveal the
  post-miss hold exists to show.
  The run ends in `advance_pending_level`, when the hold expires, not at the pick: it
  is the hold that shows the player what they missed, and a state change does not
  apply until the next frame, so ending it inline would also deal a board nobody sees.
- **`Mosaic`** — the odd-one-out asked as a pattern instead of a color. `src/wfc.rs`
  tiles the grid with pipe pieces whose edges must agree, then breaks exactly one.
  The tile set is deliberately incomplete (no dead ends, no crosses) — with a complete
  set every edge assignment would be realizable and propagation would rule nothing
  out, which is to say it would not be WFC at all. Dials: `mosaic_dimensions_for_level`
  (2x3 → 5x5) and `mosaic_violations_for_level`, which is **only ever 4 or 2**.
  That restriction is forced by the tile set, not chosen: three edges away from a piece
  with two arms or fewer is always a three-armed piece, and so is one edge away from a
  legal piece at the border. Both settings made the impostor a T every single time —
  a tell the player learns in two rounds. Difficulty past level 1 rides on board size.
  **A break of one edge is only fair against the outside of the board.** A
  disagreement belongs to the edge, not the cell, so a single interior mismatch
  implicates both neighbours equally and the game would mark one of two defensible
  answers wrong. Two or more violations make the answer "the piece wrong on more than
  one side"; one violation is allowed only where the other party is the void. The
  tests in `wfc.rs` pin this down — that is what they are for.
- **`MemoryPhase`** (Resource) — drives a `Memory` round: preview, then hidden.
  `hide_memory_board` repaints every square's `Fill` when the preview ends, which
  leaves the entities alone so the hit test and the answer reveal keep working.
  `player_interaction` refuses input while previewing, and repaints the true colors
  on a miss so a missed round still teaches.
- **`GameTimer`** (Resource) — a single `Timer`; starts paused. When it finishes,
  `tick_game_timer` transitions to `GameOverResume`. Untimed runs never expire; they
  end by running out of lives, or via "ENCERRAR PARTIDA" on the pause screen.
- **`SavedRun`** (Resource, `src/game/score/`) — the run in progress, persisted as
  `mode=score:lives`. The board is not stored: it is dealt fresh every round, so there
  is no position to restore — only a place in the curve, which the score carries. The
  lives are there for the opposite reason, being the one piece of run state the score
  cannot be derived from; without them the way to survive a bad round would be to
  close the tab. Written whenever score *or* lives change rather than on exit, because
  the way a browser game ends is a closed tab. A save with no lives left is dropped
  rather than stored, and `restore_lives` floors at one — a resumed run with zero
  could never end, since only a miss ends one. The old `mode=score` format still
  loads, resuming at a full complement. Cleared when a run finishes. The main menu
  offers it as a `CONTINUAR` card above the mode list.
- **`BestScores`** (Resource, `src/game/score/`) — per-mode personal bests, loaded at
  startup and saved on each run end. `LastRunOutcome` carries score/best/is_record to
  the game-over screen; the screen orders itself `.after(RecordOutcomeSet)` so it
  never renders the previous run's numbers.
- **`GameHistory`** (Resource) — every `LevelHistory` (clicked position, all
  `LevelColor`s, which was correct, whether it scored) plus aggregate stats
  (`levels_played`, `total_score`, `max_streak`, `total_time`). Drives both the
  history menu and the level replay.
- **`PuzzleColor`** (Component) — one per rendered piece, carrying its centre and its
  `corners` relative to that centre. The outline is stored per piece rather than
  recomputed from the current level, so a replayed round is drawn and hit-tested exactly
  as it was played. `PuzzleColorGame` is the broad marker used by `despaw_objects` (sic)
  to clear the board on `OnExit(Game)`.
- **`board::layout(min, max, columns)`** — the colour modes' board: an analytic
  *pointy-top* hex lattice. Circumradius `R` comes from the column count and the width
  (`√3·R` per column); the row count is whatever fits the height at `1.5·R` spacing, so
  **rows and columns cannot be chosen independently** — with congruent regular hexagons
  the aspect ratio of the screen decides. Odd rows hold `columns−1` cells and shift by
  half a width, which is what makes them interlock. The gap is a **uniform scale about
  the centre** (`R_draw = R − gap/√3`), not a shrink, because on a regular hexagon that
  is the only thing giving an identical vent on all six sides. Edge cells stay **whole**
  and the leftover is centred — clipping them to the frame would return pieces of
  different sizes and destroy the congruence the eye compares against. `layout` returns
  each piece's `(column, row)`, which is how `mosaic_pattern` knows who touches whom;
  `neighbours` gives the six offsets in odd-r offset coordinates. O(n), no rejection.
  **At 16 columns on a 390px phone a hexagon is ~22px, under the 48px touch guideline.**
  That is a deliberate trade — the 48px floor still applies to UI buttons.
- **`mosaic_pattern::generate`** — decides, per cell, empty or which colour group.
  Empties are **grown in veins from seeds**, not sprinkled: scattered empties read as
  noise, connected ones read as a mosaic. Groups come from a **multi-source BFS** over
  the lattice, which makes every group connected by construction. `choose_answer`
  prefers a cell surrounded by its own group and requires that group to hold at least
  two cells, so the answer always has near-twins to hide among.
- **Pieces are convex polygons, and everything downstream knows it.** `PuzzleColor`
  carries `corners` relative to the piece's centre — which is its `Transform`, unlike
  the old squares, which were spawned from their bottom-left corner. Hit testing is
  `Piece::contains` (left of every edge), and the answer reveal traces the same
  outline rather than a stand-in rectangle. `Mosaic`'s grid cells are square pieces
  built the same way, so there is one path for both.
- **`BoardGrid`** — `Mosaic` only. `grid_for_dimensions` sizes the grid its pattern was
  generated for and centers it in the window minus `HUD_RESERVED_HEIGHT`.

Round flow: `start_puzzle_level` (OnEnter Game) sizes the puzzle to the window and
fires `StartLevelEvent` → `spawn_objects` despawns the old board and lays the new
pieces out on `current_slots` (or, in `Mosaic`, on `mosaic_grid()`) →
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
`BannerEvent` for level ups, and `RevealIn` for staggered text. Streak *messages* were
removed on purpose — the HUD's "SEQ" counter and the summary's "MAIOR SEQUENCIA" stay,
because those are numbers rather than interruptions — and why losing a life is
reported by the HUD pips rather than a banner. `BannerEvent` is still only ever a
level up (the regained life is folded into that banner's text), which is why
`audio.rs` can play the level sound off it without filtering. Any genuinely new
banner type has to filter `play_level_sound`, or it will chime like a level up.
`src/interaction_animation.rs` consumes `InteractionAnimationEvent` and renders the
per-pick response: green ring and "+1" on a hit, red cross plus shake and a blinking
outline over the correct square on a miss.

The event carries `scored`, `bonus_seconds` and `correct_position` precisely so hit and
miss can look different — if you add a new outcome, extend the event rather than
inferring the result downstream.

Rendering uses **bevy_prototype_lyon**, not sprites. `Fill` and `Stroke` are no longer
components: the component is `Shape { path, fill: Option<Fill>, stroke: Option<Stroke> }`,
built with `ShapeBuilder::with(&geometry).fill(..).build()` and spawned alongside its own
`Transform`. Writing `shape.fill` / `shape.stroke` propagates to the mesh, which is how
`hide_memory_board` and the pick animations repaint. A piece is a
polygon whose `Transform` is its *centre* and whose `corners` are relative to it — not
the bottom-left origin the old squares used. Depth is handled by incrementing `z` by 0.1
per piece.

**Pointer coordinates.** Everything is top-left now and nothing is flipped.
`viewport_to_ndc` flips y itself, so `viewport_to_world_2d` wants a *top-left* origin;
both `Window::cursor_position` and `Touches` hand winit's top-left positions straight
through. `player_interaction` therefore passes either one in unchanged.

This inverted at 0.13, and it is the one change in the port that compiles clean and
fails silently: under 0.10 the cursor was bottom-left and the touch branch had to flip.
Keeping either flip against 0.19 mirrors every pick vertically with no compiler
complaint, so it was fixed in a commit of its own to keep it visible in the diff.

The background is the camera clear color, and in the color modes it *is* the answer.
`BackgroundTranstion` (`src/systems.rs`, spawned on the camera entity) holds a **path**
of colors, not a from/to pair: `sweep(from, stops, seconds)` walks it with equal time
per stop and `get_current_color` lerps piecewise along it. `ColorPuzzle::sweep()` builds
the path — the round's palette shuffled, with the answer's colour removed from the body
and appended as the final stop, so the ground never lands on it early. The pieces are
spawned in their final colors and never animated; a group vanishing is simply the
consequence of the ground reaching its colour.

**Input is accepted from frame 0.** Removing the old dead second was the point. What
`is_in_transition()` also did, and what had to be replaced rather than deleted, is stop
a double score: `spawn_objects` despawns the old board through `Commands`, which apply
at the end of the stage, so for one frame `player_interaction` still sees last round's
entities. `RoundIntro` (`LOCK_SECONDS` = 0.12, shorter than any human double-tap) is
armed at spawn and covers exactly that. The post-miss pause is `GameMode::hold_seconds()`
— 0.45s timed, 0.7s untimed — and `tick_game_timer` returns early while
`PendingLevelStart::is_holding()`, because otherwise a timed mode charged the player
twice for one mistake.

## Conventions and gotchas

- Bevy **0.19** API specifically. The idioms this tree uses, and the 0.10 forms
  they replaced (the port from 0.10 landed in one commit; anything below that
  still reads like the left-hand column is a leftover):

  | 0.10 | 0.19 |
  | --- | --- |
  | `add_system(x.in_schedule(OnEnter(S)))` | `add_systems(OnEnter(S), x)` |
  | `add_system(x)` / `add_startup_system(x)` | `add_systems(Update, x)` / `add_systems(Startup, x)` |
  | `(a, b).in_set(OnUpdate(S))` | `(a, b).run_if(in_state(S))` |
  | `.in_base_set(CoreSet::PostUpdate)` | `add_systems(PostUpdate, x)` |
  | `add_plugin(P)` / `add_state::<S>()` | `add_plugins(P)` / `init_state::<S>()` |
  | `add_event::<E>()`, `EventReader/Writer` | `add_message::<E>()`, `MessageReader/Writer`, `#[derive(Message)]` |
  | `events.iter()` / `writer.send(e)` | `events.read()` / `writer.write(e)` |
  | `Interaction::Clicked` | `Interaction::Pressed` |
  | `Style` + `Size::new(w, h)` | `Node` with `width`/`height` (and `min_*`, `column_gap`/`row_gap`) |
  | `NodeBundle`/`ButtonBundle`/`TextBundle` | component tuples; `Button` is its own marker |
  | `Text` with `sections` | `Text` (the string) + sibling `TextColor`, `TextFont`; runs are `TextSpan` children |
  | `Text2dBundle` | `Text2d` + the same sibling components |
  | `ZIndex::Global(n)` | `GlobalZIndex(n)` |
  | `app_state.0` | `*app_state.get()` |
  | `query.single()` | returns `Result` — `let Ok(x) = ... else { return; }` |
  | `viewport_to_world_2d` → `Option` | → `Result` |
  | `Camera2dBundle`, `Camera2d.clear_color` | `Camera2d` marker; `clear_color` is on `Camera` |
  | `Color::rgb`, `color.set_a(a)` | `Color::srgb`, `color.set_alpha(a)`; channels via `to_srgba()` |
  | `Time::delta_seconds` | `Time::delta_secs` |
  | `Timer::finished/percent/paused` | `is_finished`/`fraction`/`is_paused` |
  | `Input<T>` | `ButtonInput<T>` |
  | `ChildBuilder` | `ChildSpawnerCommands` |
  | `despawn_recursive()` / `despawn_descendants()` | `despawn()` (recursive by default) / `despawn_related::<Children>()` |
  | `KeyCode::G` | `KeyCode::KeyG` |
  | `AppExit` (unit) | `AppExit::Success` |
  | `Node` style consts | `Node` owns `Vec` fields, so it has a destructor and **cannot be a `const`** — these are functions |

  Do not port code from 0.10-era examples without translating it.
- **The toolchain floor is rustc 1.95** (bevy 0.19.1). An older container needs
  `rustup update stable` before anything here will build.
- **`x11` and `wayland` are no longer Bevy default features.** `Cargo.toml` turns
  both on explicitly; without at least one, a Linux build compiles and then has no
  windowing backend to open a window with.
- **`libwayland-dev` and `libxkbcommon-dev` are build dependencies on Linux now.**
  They come in through `accesskit_winit → winit`, independent of which of the two
  backend features are enabled, so they are needed even for an X11-only build.
  `.devcontainer/setup.sh` installs them.
- Event readers overwhelmingly use `events.iter().next()` and process only the first
  event per frame. That is the established pattern here; it also means bursts of
  events get dropped. Preserve it unless a bug specifically requires draining.
- Names contain typos that are load-bearing (referenced across files): `despaw_objects`,
  `BackgroundTranstion`, `HistoryButtom`, `HistoryBackButtom`, `last_interraction_event_writer`.
  Rename only deliberately and repo-wide.
- All colors, font sizes and spacing come from `src/theme.rs`. Feature `styles.rs`
  files re-export from it and keep only their own layout `Node` builders — do not
  reintroduce per-screen palettes. Note these are **functions, not consts**: `Node`
  owns `Vec` fields for grid tracks, so it has a destructor and cannot be a `const`.
  `theme::TextStyle` is likewise ours, not Bevy's — it keeps font, size and colour
  together and splits into `TextFont` + `TextColor` once, inside the builders, which
  is why the ~100 `theme::text` call sites needed no change in the port.
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
- `Cargo.lock` **is committed** — this is an executable, not a library. It used to be
  gitignored, which meant the versions floated within their semver ranges and an
  unexplained break could be an upstream release rather than a local change.
- **Four dependencies are named nowhere in `src/` and must stay anyway.** `uuid` and
  the three `getrandom` majors, all under the `cfg(target_arch = "wasm32")` target
  table, exist only to turn on features a transitive dependency needs in order to build
  for the browser. Removing them fails the wasm build; it does not fail the native one,
  so `cargo check` alone will not catch it — check with
  `cargo check --target wasm32-unknown-unknown`, which needs no GPU and no system deps.

  **There are three `getrandom` majors in the graph at once**, reached by different
  dependencies, and each asks for its browser backend differently: 0.2 wants feature
  `js`; 0.3 wants feature `wasm_js` **and** `--cfg getrandom_backend="wasm_js"`, and
  says in a `compile_error` that the feature alone is not enough; 0.4 wants `wasm_js`
  alone, the cfg requirement having been dropped again. All three must be answered or
  the build stops at whichever it reaches first. The cfg lives in
  `.cargo/config.toml` under the wasm target — **and a `RUSTFLAGS` environment variable
  overrides config `rustflags` rather than adding to them**, so setting `RUSTFLAGS` for
  any other reason silently removes it. The deploy workflow deliberately sets none.

  This one reached `master` and broke the deployed build while native stayed green,
  which is the whole argument for running the wasm check before believing a dependency
  bump. `web-sys`, in the same table, is genuinely used, by `src/storage.rs`.
  (`bevy_rapier2d`, `bevy-inspector-egui`, `lazy_static`, `wasm-bindgen` and
  `bevy_utils` really were unused, and are gone.)
- Tests live in `src/wfc.rs`, `src/board.rs`, `src/mosaic_pattern.rs` and
  `src/game/puzzle/components.rs`, and they cover generator invariants rather than Bevy
  wiring. That split is deliberate: the generators are pure and their guarantees *are*
  the game's fairness, while everything else needs a running app to mean anything.
- No `rustfmt.toml`/`clippy.toml`, and formatting in the tree is inconsistent (mixed
  indentation, trailing whitespace). Running `cargo fmt` across the repo would produce
  a huge unrelated diff — format only what you touch.
- **Never despawn a live `Button` from a system in `Update`.** The workaround is
  still in the tree — `relayout_*` and `spawn_pagination_itens` run in `PostUpdate` —
  and a browser run on 0.19 raised no `B0003` and no `RuntimeError: unreachable`. That
  is evidence the workaround *works*, not that it is unnecessary: nothing tested
  removing it. Leave it in place. The original reasoning, from Bevy 0.10: Bevy 0.10's
  `bevy_ui::accessibility::button_changed` is registered with a plain `add_system`, so
  it sits unordered in `Update` and queues an `insert(AccessibilityNode)` for every
  button it has not tagged. A despawn queued earlier in that schedule is applied
  first, and the insert then hits a dead entity: `B0003`, which is a hard panic in
  0.10 and a bare `RuntimeError: unreachable` in the browser. Screens that tear
  themselves down and rebuild (`relayout_*`, `spawn_pagination_itens`) therefore run
  in `CoreSet::PostUpdate`. There is no way to switch the a11y systems off — `bevy_ui`
  depends on `bevy_a11y` unconditionally and adds the plugin itself. Despawning on
  `OnExit` is fine: that runs in `StateTransition`, a schedule earlier in the frame.
- **Sound is played by spawning an entity** carrying `AudioPlayer(handle)` and
  `PlaybackSettings`. `AudioSink` is a **component** Bevy adds to that same entity once
  the source has loaded — it is not an asset, so there is no `Assets<AudioSink>`, no
  handle to keep alive and nothing to upgrade. Stopping a track is despawning its
  entity; one-shots use `PlaybackMode::Despawn` so they clean themselves up.
  `bevy::audio::Volume` is a type (`Volume::Linear(f32)`), not an `f32` — and it is
  *not* this crate's `audio::Volume` resource, which is the player-facing setting.
  `Cargo.toml` enables the `wav` feature — the default set is Ogg only.
- **On iOS the side switch silences the game, and nothing in the page reports it.**
  Web Audio runs under the "ambient" audio session by default, so an iPhone with the
  Ring/Silent switch set to silent plays nothing — no error, no console warning, no
  difference in the `AudioContext` state, which still reads `running`. It is
  indistinguishable from a broken build by every measurement available from inside the
  page, and it cost a real round of debugging. Check the switch before suspecting the
  code. Safari 16.4+ can override it with `navigator.audioSession.type = 'playback'`,
  but that is deliberately **not** done here: respecting the switch is what the platform
  asks of a game, and a puzzle played in public is exactly the case the switch exists
  for.
- **The browser needs the shim in `docs/index.html` to make any sound at all.** Chrome
  builds an `AudioContext` created before a user gesture in the `suspended` state and
  leaves it there until something calls `resume()` from inside a gesture handler. Bevy
  builds its context at startup, from cpal, and never calls `resume`; nothing in
  the Rust code can reach it. (Still true at 0.19 — the shim is not a 0.10 leftover.) So the page wraps the `AudioContext` constructor before
  loading the module, keeps the instances, and resumes them on the first input. Two
  details are load bearing: the script must come **before** the `import init` module,
  because the wrap has to be in place when Bevy constructs the context; and the
  listeners are on `window` in the **capture** phase, because winit binds the canvas and
  stops the event there, so a bubbling listener never runs. Without the capture flag the
  context stays suspended through every tap — measured, not assumed. Chrome still logs
  its autoplay warning at startup; that part is unavoidable.
- Sounds live in `assets/sfx/` and are **synthesised** by `tools/make_sounds.py` (pure
  stdlib, no dependencies) so the repo owns them outright — nothing to re-source, no
  licence to track. Regenerate rather than hand-editing the WAVs. That includes the
  two music loops: `theme.wav` (menu, 12s) and `round.wav` (a round, 20s), built from a
  small additive synth in the same file. `Track.add` writes with **wraparound**, so a
  chord still ringing at the end becomes the sound already playing when the loop
  restarts — without it the seam clicks once per repeat. The round track has **no
  melody** on purpose: a line with a shape to follow competes for the attention the
  board needs.
- **Sound level is one setting with steps, not a mute flag.** `audio::Volume` is 0..=4;
  0 is off, and everything the game plays is scaled by `Volume::scale()`. The pause
  screen's one button cycles down and wraps — a drag target is the wrong shape for a
  thumb. `Volume::load` **ignores** the old
  `color_puzzle.muted` key it replaced, and that is deliberate: the old control was a
  two-state toggle cheap to tap out of curiosity, so browsers have `muted=1` left in them
  from a single idle press. Honouring it started the game silent, and with the pause
  screen not repainting in the browser the reading on the button never changed either —
  the game simply appeared to have no sound. That happened to a real player. Hearing one
  round of music you did not want is a far smaller harm than a mute you cannot see.
- **Music is reconciled, not triggered.** `audio.rs::reconcile_music` computes the
  wanted track each frame from `AppState` and `Volume` and switches if it differs — one
  system covering both screen changes and the mute button, with no ordering between
  them. `Music` holds the playing track's **entity**, and stopping is despawning it —
  which lands whether or not the sink exists yet. Both 0.10 traps this used to
  document (the weak handle that detached on drop, and the retry loop for the frame or
  two before the sink asset appeared) went away with the asset-based `AudioSink`; the
  `Music::stopping` field is gone with them. One residue survives: the `AudioSink`
  *component* is still added a frame or two after the entity is spawned, so
  `follow_volume` keeps reapplying the level until it takes.
  Only *silence* goes through the reconciler; every other level
  change is applied to the live sink by `follow_volume`, because stopping and restarting
  would drop the player back to the top of the loop on every press.
- **Do not gate a label on `Res<T>::is_changed()` when the writer is a sibling in the
  same tuple.** Systems in a tuple run in an arbitrary order, so the reader can run
  *before* the writer in the very frame the value changes — and the flag is only set for
  that one frame, so the label never catches up. `update_sound_label` compares the string
  instead, which is immune to the ordering and still avoids re-laying out the text.
- **FIXED by the 0.19 port: the pause and game-over screens not repainting in the
  browser.** This was the bug that made pagination, "menu principal" and "reiniciar"
  look like the game had hung, while the ECS ran at a full 61 updates a second behind
  a frozen picture. It is gone.

  Measured, not assumed: on the 0.19 wasm build in Chromium, tapping the pause
  screen's sound button stepped the label 100% -> 75% -> 50% **on screen**, and the
  button painted its hover state. That is the exact symptom that used to be frozen.

  The diagnosis that survived the longest was right about where it lived. The last
  note on it named the render graph and wgpu's WebGL surface as the remaining
  suspects, having ruled out the UI tree by measurement — `ZIndex` in all three
  forms, `PositionType::Absolute`, the `SCRIM` colour, and a root made structurally
  identical to the main menu's. The port replaced both suspects wholesale (Bevy
  0.10 -> 0.19, wgpu 29), and the symptom went with them. No code of ours was
  changed to fix it.

  Keep the measurement advice, because it is what made the result trustworthy: force
  `preserveDrawingBuffer` in a `getContext` shim, since a WebGL canvas without it can
  hand back a stale buffer to `readPixels` *and* to a screenshot; and validate any
  observation tool against a change you know happens before trusting a negative.

- **`std::time::SystemTime::now()` panics on `wasm32-unknown-unknown`.** It is fine
  natively, so it compiles and passes tests and then takes the web build down at the line
  that calls it — which, since a wasm panic just stops the frame loop, looks exactly like
  the freeze above. Use `Res<Time>` or a `Local` counter. This cost two misleading
  experiments during the investigation of that bug.

### Visual language

The palette in `theme.rs` is the mock-up's: near-black violet ground (`BACKGROUND`),
panels a step above it (`SURFACE`, `SURFACE_RAISED`), and saturated accents —
`PRIMARY` purple for the one action a screen wants taken, `DANGER` crimson for the one
that ends a run, `SUCCESS`, `ACCENT`, `LIME`, `INFO`. Each `GameMode` owns one of them
via `accent()`.

Borders are drawn as a wrapper node painted in the border color with `HAIRLINE`
padding, containing the real panel — see `theme::outlined_style`. This dates from
Bevy 0.10, whose UI could not stroke a node. 0.19 has `BorderColor`/`Node::border`, so
the wrapper is no longer forced; it is simply what the tree still does, and collapsing
it would touch every outlined panel, so it was left alone by the port. Note that Taffy sizes the *content* box: a node at
`Percent(100)` with padding is wider than its parent, and a `Px` node's padding is
added outside its width. Both mistakes push whole screens off the edge, and both were
in this codebase.

The board pieces are deliberately flat — no stroke, no glow, unlike the mock-up's
blocks. Any outline at all would draw the hidden piece's border and hand over the
answer.

### Dead code to be aware of

- `AppState::Paused` is declared and unused — pausing goes to `History`, which doubles
  as the pause screen.
- `PuzzleColor::index` is stored but never read.
- `PIXELS_PER_METER` and `RESOLUTION` in `main.rs` are leftovers from the template.

## Git workflow

Default branch is `master`. Feature work happens on `claude/*` branches; push with
`git push -u origin <branch>`. Don't open a PR unless asked. Commit messages in the
existing history are terse ("new", "fix"); write clearer ones than that.
