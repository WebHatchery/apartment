# Second Story TODO

Active agent tasks only

**Review sizes** (used by every "Verify" below): 1280x720 normal, 1920x1080
wide, 960x540 short, 640x480 minimum landscape, and 390x844 portrait. Capture
with the `APARTMENT_CAPTURE_MANIFEST` harness (scenes in `src/game.rs`).

## UI tasks

- [ ] **UI-11 — Close browser, dense-state and recovery verification gaps.**
  **Scope:** `src/game.rs` capture scenes, `src/state/menu.rs` and
  `src/state/menu/building_select.rs`, `src/ui/career_summary.rs`,
  `src/ui/ownership_panel.rs`, `src/ui/application_panel.rs`, `game_page.json`,
  and `docs/verification/`. 
  **Problem:** verification so far is native captures and source reading — no
  live browser/touch, embedded canvas, zoom/display-scale, animation, portrait
  secondary screens, dense endgame or full-campaign check.
  **Action:** capture current states at every review size, replacing equivalent
  PNGs in `docs/verification/`. Inspect building-picker density with many
  unlocks, career/demo endings, long applications, ownership sale/buyback and
  purchase review. Fix reproduced issues within the relevant task above; record
  new findings with evidence. Exercise title → new game/continue → tutorial →
  core management → month → event → pause save/load → exit to title, plus
  failure recovery, using visible touch controls only. Verify coordinate
  conversion after resize/display scaling and in the embedded browser.
  **Accept:** each changed screen passes UI_STYLE §9 with named screenshots,
  tested interactions and stated limitations; prices, warnings, cancellation
  and recovery stay accessible in dense states.

## Engineering backlog

- [ ] Reshape the binary into a testable crate: extend the existing `src/lib.rs`
  geometry seam, keep `main.rs` as the runtime entry point, and migrate the 51
  `src/**/tests.rs` suites into `tests/` integration suites through intentional
  public seams. Preserve useful coverage; for suites with more than five cases
  per feature, use table-driven assertions or document why cases stay distinct.

- [ ] Make JSON the single runtime source of truth for content, configuration,
  and player-facing text: wire `assets/text_strings.json` through the toolkit
  loader with typed semantic validation, then migrate hardcoded UI/status text
  and duplicate archetype/config defaults, keeping only intentional schema
  fallbacks.

- [ ] Extend `tests/code_standards.rs` or CI with enforceable checks for module
  docs, forbidden `mod.rs` files, in-source tests, undocumented `#[allow]`
  attributes, file/function size signals, and direct game-data parsing so the
  standards do not regress silently.

## Validation for every slice

`cargo fmt -- --check`, `cargo test`, strict all-target/all-feature Clippy, the
800-line gate, and `.\publish.ps1` (no parameters) for meaningful changes.
Preserve `catalog_thumbnail.png`; commit each independently useful slice.
