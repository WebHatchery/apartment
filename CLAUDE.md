# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**Second Story** — a cozy building-management game built in Rust with
`macroquad` and the shared, path-local `macroquad-toolkit` crate
(`../macroquad-toolkit`, a sibling directory that must be present). You manage
a neglected apartment block over 36 months: repairs, upgrades, tenants, rent,
missions, and reputation, unlocking harder properties as you go.

The repository folder, Cargo package, capture environment prefix, deployment
slug, and save namespace deliberately retain their legacy `apartment` /
`apartment_manager` identifiers until the repository and folder migrate. Do
not rename the save namespace without a migration path for existing saves.

## Commands

There is no local wrapper for the everyday loop — use `cargo` directly from this directory:

- Build: `cargo build` (native) / `cargo build --release`
- Run the game: `cargo run`
- Tests: `cargo test` — run a single test with `cargo test <name>` (e.g. `cargo test rent`)
- Format check: `cargo fmt -- --check` (CI enforces this)
- Lint: `cargo clippy --all-targets --all-features -- -D warnings` (CI treats warnings as errors)
- WebGL build: `cargo build --release --target wasm32-unknown-unknown` — the wasm link flags come from the workspace `.cargo/config.toml`; do not set `RUSTFLAGS`, it replaces that list and forces a full rebuild of every wasm dependency

`publish.ps1` is a thin wrapper that delegates to the workspace-root `../publish.ps1` for cross-platform builds and deployment (`-WindowsOnly`, `-WebGLOnly`, `-Production`, `-DryRun`, etc.). Per `AGENTS.md`, `.\publish.ps1` (no params) is the sanctioned end-to-end validation path after meaningful changes — but for tight iteration prefer `cargo clippy` + `cargo test`, which is exactly what CI (`.github/workflows/rust-ci.yml`) runs.

The itch.io release is separate: `itch.json` identifies the existing
`kalaith/second-story` page and its stable `html5`/`windows` channels. Run
`.\publish.ps1` first, then use `.\publish-itch.ps1 -DryRun`,
`.\publish-itch.ps1 -Preview`, and finally `.\publish-itch.ps1` when the
channel diff is correct. `.\publish-itch.ps1 -Status` only queries Butler and
does not upload.

## Architecture

The whole app is a single `#[macroquad::main]` loop in `main.rs` calling `Game::update()` then `Game::draw()` each frame (`game.rs`). `Game` owns three things: a `GameState` enum, the loaded `GameConfig`, and the `AssetManager`.

**State machine.** `GameState` (`state.rs`) is a two-variant enum: `Menu(MenuState)` and `Gameplay(GameplayState)`. Each state's `update()` returns an `Option<StateTransition>`; `Game::transition()` swaps the state. All real gameplay lives in `GameplayState` (`state/gameplay.rs`), a large serde-serializable struct that is the save game. Its logic is split across sibling files by responsibility — do not put it all in one file:
- `gameplay_actions.rs` — dispatches `UiAction` intents and city actions
- `gameplay_turn.rs` — monthly turn advancement (the core simulation step)
- `gameplay_effects.rs` — applying narrative event effects
- `gameplay_views.rs` — top-level drawing dispatch per `ViewMode`

**UI is a pure view layer.** The `ui/` module renders with macroquad immediate mode and **never mutates game state**. Panels return `UiAction` intent enums (`ui.rs`) which `gameplay_actions.rs` interprets. When adding an interaction, add a `UiAction` variant and handle it in the dispatcher rather than reaching into state from a panel. `Selection` and `ViewMode` track what the detail panel / screen is showing.

**Simulation.** `simulation/` is the time engine: `advance_tick` (`tick.rs`) produces a `TickResult` each month, driving decay (`decay.rs`), random/world events, and win/loss checks (`win_condition.rs`). Keep this deterministic where practical; isolate RNG behind helpers or state-owned RNG.

**Domain modules** each own one concept and mirror the shape `foo.rs` + `foo/*.rs`: `building/` (apartments, upgrades, ownership), `tenant/` (archetypes, applications, happiness, matching, vetting), `economy/` (funds, ledger, rent, costs), `city/` (neighborhoods, market — multi-building layer), `consequences/` (gentrification, regulations, relationships), `narrative/` (events, missions, mail, dialogue, tutorial, achievements). Many features are labeled "Phase 3/4/5" — that reflects incremental build-out, not separate build targets.

**Data-driven design is a hard rule here.** Balance values, upgrades, tenant archetypes, events, neighborhoods, achievements, text strings, etc. live as JSON in `assets/` and are loaded via serde. `data/config.rs::load_config()` embeds these with `include_str!` for the wasm build and reads from disk (falling back to embedded) for native — so a config change is picked up at runtime natively but requires a rebuild for wasm. Prefer editing the relevant `assets/*.json` over hardcoding constants in Rust. Runtime save files (`savegame.json`, `player_progress.json`) live at the repo root, distinct from the `assets/` config.

## Conventions (from AGENTS.md / CODE_STANDARDS.md)

- Keep every `.rs` file under 800 lines. A file approaching the limit is a signal to extract a cohesive responsibility into a nearby module — not to strip whitespace. If a change would push a touched file over, do the restructure as part of the task.
- Use named module filenames (`foo.rs`, `foo/bar.rs`). Do **not** create new `mod.rs` files.
- Reach for `macroquad-toolkit` first for input, UI widgets, assets, camera, colors, sprites, event bus. Treat a missing runtime/rendering/input capability as a toolkit upgrade before writing a project-local alternative; only diverge when the need is genuinely game-specific.
- Keep drawing separate from state mutation. Avoid hardcoded absolute positions not tied to the fixed virtual resolution; the window is resizable (1280×720 default) and must stay playable at common desktop/browser sizes.
- No unused code — delete unused fields/functions rather than `_`-prefixing them (except where a trait signature requires it).
- Match existing style; avoid broad refactors bundled into focused changes. Don't add dependencies unless they remove real complexity or match an established pattern.
