- [ ] Reshape the binary into a testable crate: add `src/lib.rs`, keep `main.rs`
  as the runtime entry point, and migrate the 52 `src/**/tests.rs` suites and
  remaining in-source test modules into `tests/` integration suites through
  intentional public seams. Preserve useful coverage and review the suites
  with more than five cases per major feature, using table-driven assertions
  or documenting why distinct cases remain.

- [ ] Add a short `//!` purpose comment to each of the 100 production Rust
  files that currently lacks module-level documentation, including child
  modules introduced during the test and size migrations.

- [ ] Split the files at the 600-line refactor signal into cohesive modules:
  `src/sim_harness.rs`, `src/ui/workspace_views.rs`,
  `src/state/gameplay_actions.rs`, `src/ui/city_view.rs`,
  `src/state/gameplay.rs`, `src/consequences/relationships.rs`,
  `src/game.rs`, and `src/narrative/stories.rs`. Keep every resulting Rust
  file comfortably below the 800-line hard limit without cosmetic compression.

- [ ] Refactor long and over-arity paths such as `GameplayState::process_action`,
  `GameTick::process`, `MissionManager::evaluate_active`,
  `TenantNetwork::tick`, and the capture-scene dispatcher around context or
  result structs. Remove the broad `too_many_arguments`, `large_enum_variant`,
  `module_inception`, and `non_snake_case` allowances where possible, document
  any remaining narrow allowance, and remove unused parameters such as the
  city argument in `draw_neighborhood_cell`.

- [ ] Make JSON the single runtime source of truth for content, configuration,
  and player-facing text: wire `assets/text_strings.json` through the toolkit
  loader with typed semantic validation, then migrate hardcoded UI/status text
  and duplicate archetype/config defaults while retaining only intentional
  schema fallbacks.

- [ ] Standardize interactive controls on the toolkit's release semantics.
  Audit direct `is_mouse_button_pressed` checks and `ButtonTrigger::Press`,
  keep press-only behavior only where immediate feedback is intentional, and
  add regression coverage for touch/click activation without duplicate actions.

- [ ] Extend `tests/code_standards.rs` or CI with enforceable checks for module
  docs, forbidden `mod.rs` files, in-source tests, undocumented `#[allow]`
  attributes, file/function size signals, and direct game-data parsing so the
  updated standards do not regress silently.
