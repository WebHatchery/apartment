# Second Story TODO

Active agent tasks only

**Review sizes** (used by every "Verify" below): 1280x720 normal, 1920x1080
wide, 960x540 short, 640x480 minimum landscape, and 390x844 portrait. Capture
with the `APARTMENT_CAPTURE_MANIFEST` harness (scenes in `src/game.rs`).

## UI tasks

- [ ] **UI-05 — Keep selected rooms visible and put their immediate decisions first.**
  **Scope:** `src/screen_layout.rs::BuildingLayout::new`,
  `src/ui/building_view.rs::{draw_building_view,cutaway_metrics}`,
  `src/ui/apartment_panel.rs`, `src/ui/apartment_panel_sections.rs`,
  `src/ui/tenant_panel.rs`, `src/ui/hallway_panel.rs`, and selection handling in
  `src/state/gameplay_actions.rs`.
  **Problem:** at 390x844, selecting a unit shows upper-floor rooms instead of
  the selected one, and the bottom inspector takes most of the remaining space.
  The inspector opens on a repair plus a long stat stack, with tenant and
  actions further down; repair appears both at the top and in the upgrade list.
  The compact hallway view shows staff while the lobby is out of view.
  **Change:** scroll/reframe to the selected unit or lobby when the panel opens
  or the canvas resizes, keeping useful scale and visible Up/Down controls.
  Lead with identity/vacancy, relevant condition/issue, rent and the current
  action with price/affordability. Give repair one home; disclose design, size,
  kitchen, lighting, quality and advanced tenant detail as needed. Group staff
  and upgrades behind explicit disclosure instead of a continuous stat wall.
  Keep applications, ownership, listing and sold-unit routes accessible.
  **Accept:** players see which place they are acting on and its immediate
  consequence without scrolling through unrelated facts. Default desktop
  composition remains world-first; closing restores a useful building view.
  **Verify:** all review sizes: top/bottom floors, lobby, tall building, vacancy,
  sold unit, unaffordable repair, applications and ownership. Tap target → action
  → result → Close, scroll both regions, resize, and reselect; confirm draw and
  hit bounds agree and no covered room receives the same input.

- [ ] **UI-06 — Make City and property comparison readable before adding decoration.**
  **Scope:** `src/ui/city_view.rs::{draw_city_map,draw_neighborhood_cell,draw_portfolio_panel,draw_market_panel,draw_purchase_confirmation}`
  and `src/ui/city_view_widgets.rs`.
  **Problem:** City always halves the canvas, then splits the map into a 2x2
  grid. Small tiles layer labels, district type and reputation over busy art;
  portfolio names and occupancy truncate.
  **Change:** prioritize neighborhood selection, then the filtered property
  comparison, switching/stacking at small widths. Reveal district statistics
  on selection rather than overlaying every fact on every tile. Give property
  price, affordability, occupancy and purchase consequences a readable home;
  preserve filters, Enter, purchase review/cancel and clear return routes.
  **Accept:** players can identify districts and compare complete purchase terms
  without four equally loud tiles plus a competing action panel; no lost filters.
  **Verify:** 1280x720, 960x540, 640x480 and 390x844 with populated/empty filtered
  markets, multiple owned properties and long names. Touch district → listing →
  review → cancel/buy → enter property; verify hit targets after resize.

- [ ] **UI-07 — Merge monthly progression and activity into one brief, retrievable result.**
  **Scope:** `src/state/gameplay_turn.rs::{end_turn,spawn_tick_feedback}`,
  `src/state/gameplay_input.rs`, `src/state/gameplay_views.rs`,
  `src/ui/notifications.rs::{draw_notifications,draw_activity_drawer}`, and
  `src/economy/ledger.rs`.
  **Problem:** each rent event emits floating text around screen center; the
  footer keeps the latest event indefinitely on wider canvases; history shows
  at most five truncated events. There is no dedicated month-result view.
  **Change:** gather the closed month's ledger report and meaningful tenant/
  building events into a brief, dismissible summary; use the ledger's monthly
  report rather than computing separate totals. Replace per-rent center bursts
  and stale footer prose with temporary localized/aggregate feedback. Keep a
  quiet history control with readable detail after feedback ends. Persistent
  warnings describe unresolved current issues, not old events. Explicitly order
  summary, mandatory choices, tutorials, campaign outcome and demo ending, and
  guard End Month against repeated input during the sequence.
  **Accept:** a player understands what changed in seconds, can inspect it later,
  and advances exactly one intended month; calm play returns to the building.
  **Verify:** at all review sizes exercise rapid repeated taps, rent/missed rent,
  repairs/upgrades, arrivals/departures, portfolio income, mandatory event plus
  tutorial, loss, month 36 and demo completion. Check totals against the ledger
  and history after dismissal.

- [ ] **UI-08 — Make coaching accurate, dismissible and reopenable.**
  **Scope:** `src/state/gameplay_views.rs::draw_tutorial_coach`,
  `src/state/gameplay_pause.rs` (pause menu), `src/state/tutorial_system.rs`,
  `src/narrative/tutorial.rs`, and relevant `UiAction` routes. After UI-05.
  **Problem:** coach anchors are guessed screen positions rather than the
  target's rendered bounds; the vacancy step anchors at `screen_width() - 68`,
  near Menu rather than End Month. The pause menu has no Help/reopen option.
  Milestone coaching stays visible until progression and hides entirely while
  a selection is open.
  **Change:** derive highlights from shared target geometry; add a quiet Help
  option (e.g. in the pause menu) that dismisses/reopens current instructions
  without resetting tutorial or saves. Teach the exact current labels/touch
  paths, and let brief contextual help accompany the required panel action
  without obscuring it.
  **Accept:** completed prose disappears, returning players can recover guidance,
  and each highlight identifies the action actually described.
  **Verify:** first game and resumed tutorial at all review sizes; dismiss/reopen,
  lobby repair, listing, End Month, filtered applications, damaged-unit repair,
  selection/Close and scrolling. Ensure coach taps cannot activate rooms beneath.

- [ ] **UI-09 — Extend tenant identity and passage of time using existing state.**
  **Scope:** `src/ui/resident_sprite.rs::{style_for,animated_pose}`,
  `src/ui/building_view/room_story.rs`, `src/ui/building_view.rs::draw_sunset_backdrop`,
  `src/ui/building_view/condition_art.rs`, assets and presentation configuration.
  After UI-05 and UI-07.
  **Problem:** resident styling and pose cycling exist, but same-design rooms
  repeat their furnishings, room-story inputs are only apartment features, and
  the exterior uses a fixed sunset. (Residents, wear and upgrade art are done —
  don't redo them.)
  **Change:** add restrained identity/archetype/tenure-based dressing, small
  ambient behaviors and move-in/out feedback, with decorative randomness
  separate from simulation RNG. Add seasonal lighting/background variation
  tied to campaign time. Fix conspicuous base-art/overlay joins and baked-in
  wear that contradicts repaired state; never imply independently repairable
  lamps or invented fixture/history systems. Consolidate obsolete LayoutConfig
  tuning instead of creating a second layout authority.
  **Accept:** occupants and earned improvements are visibly distinct; Month 1
  versus later months communicates care and time without new stats, panels,
  simulation mechanics, currencies or save-format changes.
  **Verify:** same-seed worn/repaired comparisons, same-design different tenants,
  vacancy/move-in/out, upgrades and seasons at 1280x720, 1920x1080 and 390x844;
  small-room silhouettes, labels and resident selection stay clear, and
  cosmetic changes do not alter simulated results.

- [ ] **UI-10 — Finish shared readability, touch feedback and restrained motion.**
  **Scope:** `src/ui/theme.rs`, `src/ui/widgets.rs`, `src/ui/common.rs`, panel
  drawing/input in `src/state/gameplay_views.rs`, and the screens above.
  After UI-05–UI-09; polish, not another redesign.
  **Problem:** bordered buttons, panel headings, stat labels and orange
  scrolling/navigation controls compete with decisions in selected views. The
  selection Close button is 34px high while most actions are 40px; real touch
  usability is unverified.
  **Change:** remove residual nested borders/repeated labels, use spacing and
  a coherent readable type scale, keep full decision text accessible, and make
  selected/navigation/paging treatment quieter than consequential actions.
  Make essential controls at least 44 CSS px after browser scaling. Add subtle
  selection/action feedback and optional panel transitions only with
  synchronized draw/hit bounds and reduced-motion support. Share release
  activation with the engineering task below rather than duplicating it.
  **Accept:** no tiny text, invisible gestures or hover-only help; every
  necessary action is readable, touchable and acknowledged, with at most 2–3
  strong attention regions during ordinary play. Pixel art remains coherent.
  **Verify:** every review size and browser display scaling; tap edges of Close,
  repair, pagination and choice controls, drag away before release, resize during
  transition, and check reduced motion. No double action or mismatched hit area.

- [ ] **UI-11 — Close browser, dense-state and recovery verification gaps.**
  **Scope:** `src/game.rs` capture scenes, `src/state/menu.rs` and
  `src/state/menu/building_select.rs`, `src/ui/career_summary.rs`,
  `src/ui/ownership_panel.rs`, `src/ui/application_panel.rs`, `game_page.json`,
  and `docs/verification/`. Run alongside each task above; finish after UI-10.
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

- [ ] Add a short `//!` purpose comment to each of the 62 production Rust files
  that lack module-level documentation, including child modules created by the
  test and size migrations.

- [ ] Split the files at the 600-line refactor signal into cohesive modules:
  `src/ui/workspace_views.rs` (at the 800 limit — split before any edit),
  `src/sim_harness.rs`, `src/game.rs`, `src/state/gameplay_actions.rs`,
  `src/ui/workspace_tasks.rs`, `src/ui/city_view.rs`, `src/state/gameplay.rs`,
  `src/consequences/relationships.rs`, and `src/narrative/stories.rs`. Keep
  every resulting file comfortably below 800 lines without cosmetic compression.

- [ ] Refactor long and over-arity paths such as `GameplayState::process_action`,
  `GameTick::process`, `MissionManager::evaluate_active`,
  `TenantNetwork::tick`, and the capture-scene dispatcher around context or
  result structs. Remove the broad `too_many_arguments`, `large_enum_variant`,
  `module_inception`, and `non_snake_case` allowances where possible, document
  any remaining narrow allowance, and remove unused parameters such as the
  `_city` argument in `draw_neighborhood_cell`.

- [ ] Make JSON the single runtime source of truth for content, configuration,
  and player-facing text: wire `assets/text_strings.json` through the toolkit
  loader with typed semantic validation, then migrate hardcoded UI/status text
  and duplicate archetype/config defaults, keeping only intentional schema
  fallbacks.

- [ ] Standardize interactive controls on the toolkit's release semantics.
  Audit the direct `is_mouse_button_pressed` checks (including the title menu,
  settings page and pause-settings routing) and `ButtonTrigger::Press`; keep
  press-only behavior only where immediate feedback is intentional, and add
  regression coverage for touch/click activation without duplicate actions.

- [ ] Extend `tests/code_standards.rs` or CI with enforceable checks for module
  docs, forbidden `mod.rs` files, in-source tests, undocumented `#[allow]`
  attributes, file/function size signals, and direct game-data parsing so the
  standards do not regress silently.

## Validation for every slice

`cargo fmt -- --check`, `cargo test`, strict all-target/all-feature Clippy, the
800-line gate, and `.\publish.ps1` (no parameters) for meaningful changes.
Preserve `catalog_thumbnail.png`; commit each independently useful slice.
