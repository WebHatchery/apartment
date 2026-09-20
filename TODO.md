# Second Story TODO

## UI_STYLE review — 2026-09-20

This merges the remaining work in `docs/UI_UX_CONTINUATION.md` and
`docs/UI_UX_OVERHAUL_BRIEF.md` with the updated shared `UI_STYLE.md` review.
Implement in the order below: composition and access before presentation polish.
Preserve the economy, tenant rules, 36-month campaign, demo limits, saves, warm
pixel art, essential prices, warnings, and visible recovery controls.

### Evidence and preserved completion history

Read `AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`,
`GAME_DEVELOPMENT_GUIDE.md`, `README.md`, and both UI planning documents.
No `PROJECT_AGENTS.md` or separately named GDD was found; the README and overhaul
brief supply the game intent. Inspected shell, selection, layout/camera, panel,
workspace, overlay, tutorial, turn, menu, and resident rendering code.

Existing captures visually inspected (no new captures or live play in this audit):

- 1280x720: `docs/verification/ui_showcase.png` and
  `ui_tutorial_coach_showcase.png` in the same directory.
- 1920x1080: `docs/verification/ui_showcase_wide.png`.
- 960x540: `docs/verification/ui_hallway_compact.png` and `ui_inbox_compact.png`.
- 390x844: `docs/verification/ui_building_portrait.png`, `ui_unit_portrait.png`,
  and `ui_applications_portrait.png`.
- 640x480 historical captures: `docs/verification/ui_finances_tiny.png`,
  `ui_city_tiny.png`, and `ui_tasks_tiny.png`. These August captures show the old
  header; do not treat them as current shell evidence. September applications
  and tutorial-coach captures also predate removal of world condition meters.
  Use them only for the relevant retained panel/coach composition.

Evidence labels below distinguish **visual + code**, **code**, and **planned
continuation**. Code-derived overflow is not a browser interaction result.
README does not yet declare supported canvas sizes. Use 1280x720 normal,
1920x1080 wide, 960x540 short, 640x480 minimum-landscape candidate, and 390x844
portrait candidate until task UI-01 records a validated support contract.

- [x] Building-first layout slice (recorded 2026-09-19): no default inspector,
  shared side/bottom context bounds, five main destinations, direct unit/lobby
  selection, Details route, visible Close and tower scrolling, and matching
  world draw/input clipping. Keep these gains; do not reimplement Phase 1.
- [x] Visual-state slice (recorded 2026-09-19): deterministic wear, shared-space
  condition art, existing upgrade layers, world meter removal, and immediately
  accessible priced unit repair. Exact condition remains in the inspector.
- [x] Preserve the prior validation record: continuation reports 160 passing
  tests, one ignored test, strict Clippy, native captures at four sizes, and
  successful default Windows/WebGL Preview publishing; local Project Roost
  tracking was unavailable. These are historical results, not rerun here.

### Verified findings and merged implementation tasks

- [ ] **UI-04 — Reflow People, Inbox and Tasks into readable decision views.**
  **Scope:** `src/ui/workspace_views.rs::draw_tenants_view`,
  `src/ui/workspace_inbox.rs::{draw_inbox_view,draw_selected_mail,draw_dialogue}`,
  and `src/ui/workspace_tasks.rs::draw_tasks_view`. Depends on UI-01; share the
  responsive approach established in UI-03 without forcing identical screens.
  **Evidence: code + historical visuals; current Inbox compact visual.** People
  reserves a 220px side area and Inbox a 260px list even in portrait; Tasks keeps
  a 62/38 split. Small columns truncate identity, request outcomes and content.
  Inbox also places the first pending conversation beneath independently selected
  mail, making unrelated messages compete for the same reading area.
  **Change:** make roster/mail/request lists the initial focus with a labelled
  detail/back path on small screens. Show one selected message or conversation
  at a time; keep unresolved conversations explicitly discoverable. Defer People
  aggregate statistics; let missions and resident requests switch or stack with
  deliberate scrolling. Remove redundant headings/cards and static instruction
  subtitles. Reveal full names, request costs/rewards/deadlines and consequences
  before committing; preserve Tasks & requests through Inbox and its attention cue.
  **Accept:** one comparison or decision dominates; truncation never conceals an
  action's consequences, and returning restores selection/page context.
  **Verify:** all review sizes with empty and multiple-page lists, long names,
  letters plus unrelated dialogue, multiple requests and missions. Touch Inbox →
  Tasks → request → decision → back; People → resident → unit; every pager.

- [ ] **UI-05 — Keep selected rooms visible and put their immediate decisions first.**
  **Scope:** `src/screen_layout.rs::BuildingLayout::new`,
  `src/ui/building_view.rs::{draw_building_view,cutaway_metrics}`,
  `src/ui/apartment_panel.rs`, `src/ui/apartment_panel_sections.rs`,
  `src/ui/tenant_panel.rs`, `src/ui/hallway_panel.rs`, and selection handling in
  `src/state/gameplay_actions.rs`. Depends on UI-01.
  **Evidence: visual + code.** The 390x844 Unit 2B capture shows upper-floor
  rooms instead of the selected room; the bottom inspector consumes most of
  the remaining workspace. Its initial view is a repair plus a long stat stack,
  with tenant/actions further down. Repair appears both at the top and in the
  upgrade list. Compact hallway view shows staff while its lobby is out of view.
  **Change:** scroll/reframe to the selected unit or lobby when the panel opens
  or the canvas resizes, retaining useful scale and visible Up/Down controls.
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
  and `src/ui/city_view_widgets.rs`. Depends on UI-01 and UI-02's overlay approach.
  **Evidence: code + historical visual.** City always divides the canvas in
  half, then splits the map into a 2x2 grid. Small tiles layer labels, district
  type and reputation over busy art; portfolio names and occupancy truncate.
  **Change:** prioritize neighborhood selection then the filtered property
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
  `src/economy/ledger.rs`. Depends on UI-01/UI-02; absorbs continuation time-summary
  work and quiet-notification polish. **Evidence: code + planned continuation.**
  Each rent event emits floating text around screen center; the footer retains
  the latest event indefinitely on wider canvases. History shows at most five
  truncated events. There is no concise dedicated month-result presentation.
  **Change:** gather actual completed-turn transactions and meaningful tenant/
  building events into a brief, dismissible summary; do not estimate totals or
  count repairs twice. Replace per-rent center bursts and stale footer prose
  with temporary localized/aggregate feedback. Keep a quiet history control
  with readable detail for important results after feedback ends. Persistent
  warnings describe unresolved current issues, not old events. Explicitly order
  summary, mandatory choices, tutorials, campaign outcome and demo ending, and
  guard End Month against repeated input during the sequence.
  **Accept:** a player understands what changed in seconds, can inspect it later,
  and advances exactly one intended month; calm play returns to the building.
  **Verify:** at all review sizes exercise rapid repeated taps, rent/missed rent,
  repairs/upgrades, arrivals/departures, portfolio income, mandatory event plus
  tutorial, loss, month 36 and demo completion. Check totals against the ledger
  and history after dismissal. Test turn ordering through intentional library
  seams, following the existing test-migration task before adding coverage.

- [ ] **UI-08 — Make coaching accurate, dismissible and reopenable.**
  **Scope:** `src/state/gameplay_views.rs::{draw_tutorial_coach,draw_pause_menu_overlay}`,
  `src/state/tutorial_system.rs`, `src/narrative/tutorial.rs`, and relevant
  `UiAction` routes. Depends on UI-01/UI-05; retain completed contextual coaching.
  **Evidence: code; historical coach visual.** Coach anchors are guessed screen
  positions rather than the target's rendered bounds. In particular the listed
  vacancy step anchors at `screen_width() - 68`, near Menu rather than End Month.
  Pause currently offers no Help/reopen route; milestone coaching stays visible
  until progression and hides entirely while a selection is open.
  **Change:** derive highlights from shared target geometry; provide a quiet
  visible Help route and dismiss/reopen current instructions without resetting
  tutorial or saves. Teach the exact current labels/touch paths, and let brief
  contextual help accompany the required panel action without obscuring it.
  **Accept:** completed prose disappears, returning players can recover guidance,
  and each highlight identifies the action actually described.
  **Verify:** first game and resumed tutorial at all review sizes; dismiss/reopen,
  lobby repair, listing, End Month, filtered applications, damaged-unit repair,
  selection/Close and scrolling. Ensure coach taps cannot activate rooms beneath.

- [ ] **UI-09 — Extend tenant identity and passage of time using existing state.**
  **Scope:** `src/ui/resident_sprite.rs::{style_for,animated_pose}`,
  `src/ui/building_view/room_story.rs`, `src/ui/building_view.rs::draw_sunset_backdrop`,
  `src/ui/building_view/condition_art.rs`, assets and presentation configuration.
  Depends on UI-05/UI-07. **Evidence: planned continuation + visual/code baseline.**
  Stable resident styling and pose cycling already exist; same-design rooms
  still repeat their furnishings, room-story inputs are apartment features,
  and the exterior uses a fixed sunset. Do not treat these as missing residents
  or redo completed wear/upgrade art.
  **Change:** add restrained identity/archetype/tenure-based dressing, small
  appropriate ambient behaviors and move-in/out feedback, with decorative
  randomness separate from simulation RNG. Add seasonal lighting/background
  variation tied to campaign time. Fix conspicuous base-art/overlay joins and
  baked-in wear where they contradict repaired state; never imply independently
  repairable lamps or invented fixture/history systems. Consolidate obsolete
  LayoutConfig tuning deliberately instead of creating a second layout authority.
  **Accept:** occupants and earned improvements are visibly distinct; Month 1
  versus later months communicates care and time without new stats, panels,
  simulation mechanics, currencies or save-format requirements.
  **Verify:** same-seed worn/repaired comparisons, same-design different tenants,
  vacancy/move-in/out, upgrades and seasons at 1280x720, 1920x1080 and 390x844;
  confirm small-room silhouettes, labels and selecting residents' units remain
  clear, and cosmetic changes do not alter simulated results.

- [ ] **UI-10 — Finish shared readability, touch feedback and restrained motion.**
  **Scope:** `src/ui/theme.rs`, `src/ui/widgets.rs`, `src/ui/common.rs`, panel
  drawing/input in `src/state/gameplay_views.rs`, and the screens above.
  Depends on UI-01–UI-09; absorbs remaining continuation polish, not another
  redesign. **Evidence: visual + planned continuation.** Bordered buttons,
  panel headings, stat labels and orange scrolling/navigation controls still
  compete with decisions in selected views. Existing Close is 34px high while
  most actions are 40px; real touch usability has not been verified here.
  **Change:** remove residual nested borders/repeated labels, use spacing and
  a coherent readable type scale, keep full decision text accessible, and make
  selected/navigation/paging treatment quieter than consequential actions.
  Establish usable touch sizes after browser scaling (target at least 44 CSS px
  for essential controls). Add subtle selection/action feedback and optional
  panel transitions only with synchronized draw/hit bounds and reduced motion.
  Reuse toolkit capabilities; share release activation with the retained
  engineering task below rather than duplicating an input migration.
  **Accept:** no tiny-text workaround, invisible gesture or hover-only help;
  every necessary action is readable, touchable and acknowledged, with at most
  2–3 strong attention regions during ordinary play. Pixel art remains coherent.
  **Verify:** every review size and browser display scaling; tap edges of Close,
  repair, pagination and choice controls, drag away before release, resize during
  transition, and check reduced motion. No double action or mismatched hit area.

### Further inspection and completion gate — not verified defects

- [ ] **UI-11 — Close browser, dense-state and recovery evidence gaps.**
  **Scope:** `src/game.rs` capture scenes, `src/state/menu.rs`,
  `src/ui/career_summary.rs`, `src/ui/ownership_panel.rs`,
  `src/ui/application_panel.rs`, `game_page.json`, and `docs/verification/`.
  Run alongside each affected task and finish after UI-10. **Evidence gap:**
  this audit used existing native PNGs and source; no live browser/touch,
  embedded canvas, zoom/display-scale, animation, current portrait secondary
  screens, dense endgame or full campaign verification was performed. No
  literal template demo copy was found in the inspected screens; dashboard-like
  composition alone is not evidence of copied template code.
  **Action:** capture current supported states, replacing equivalent PNGs
  directly in `docs/verification/`; inspect all normal/minimum sizes from UI-01,
  plus wide/short layouts. Inspect main-menu unlocked-property density, career/
  demo endings, long applications, ownership sale/buyback and purchase review
  before asserting additional defects. Fix reproduced issues within the relevant
  task above; record newly distinct findings with evidence instead of speculation.
  Exercise start/load → tutorial → core management → month → event → save/return,
  plus failure recovery using visible touch controls only. Verify coordinate
  conversion after resize/display scaling and actual embedded-browser dimensions.
  **Accept:** each changed screen passes UI_STYLE §9 with named screenshots,
  tested interactions and explicit remaining limitations; required prices,
  warnings, cancellation and recovery are accessible in dense states.
  **Validation:** after implementation, run formatting, appropriate tests and
  strict all-target/all-feature Clippy, the 800-line gate, and `./publish.ps1`
  without parameters. Preserve `catalog_thumbnail.png`; report publication
  result/blocker and commit each independently useful slice. This planning-only
  audit does not itself change game behavior or require publishing.

## Existing engineering backlog (retained and reconciled)

- [ ] Reshape the binary into a testable crate: extend the existing `src/lib.rs`
  geometry seam, keep `main.rs`
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
