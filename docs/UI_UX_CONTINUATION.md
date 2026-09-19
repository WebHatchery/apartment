# Second Story UI/UX continuation guide

## Scope and intent

Read `docs/UI_UX_OVERHAUL_BRIEF.md` for the user's full brief. This overhaul makes the building the main interface while preserving existing gameplay, economy, tenant rules, and the 36-month campaign. Keep the warm, imperfect pixel-art direction.

The user authorized the first implementation slice and requested these instructions for a future agent. Remaining phases are a roadmap; do not assume that all have already been implemented.

## Completed first slice

- Desktop header is one row at widths of 1180 px and above. Smaller layouts wrap navigation; portrait layouts also separate status from time controls.
- Primary navigation: Building, People, Finances, City, Inbox. Inbox carries an attention marker for unread mail or pending tasks. Tasks remain reachable through the Tasks & requests button inside Inbox.
- No default inspector. With no selection, the cutaway gets the full workspace width. The starter building and lobby fit at 1280 x 720 without scrolling.
- Select apartments and the lobby directly. The building name / Details control opens overall building information. Applications, shared spaces, Ownership, and Finances are reachable from building details. Listed vacancies retain their filtered View Applications action.
- All building context panels use the same bounds. Desktop panels stay 340-400 px wide. Below 760 px they become bottom panels, leaving a scrollable part of the building visible above them.
- Every selection has a visible Close control. Clicking empty space in the cutaway clears selection. Returning to Building from navigation also clears it.
- Large buildings retain visible Up/Down controls. Rendering and world input are clipped to the same world viewport; panels do not overlap that viewport.
- Tutorial coaching now targets the lobby in the world and explains the relocated application route. Coaching hides during selection so it does not obscure the context panel. Clicking the coach or open activity drawer does not activate covered building actions.
- Removed the old panel tween because its visual offset disagreed with the shared bounds. Panel transitions are deferred to polish and must move rendering and input together.
- No art assets, economic rules, or simulation progression were changed. Save serialization remains compatible with old selection variants; Building is an additional selection variant. Removed tween state was already excluded from saves.

## Code map

- `src/screen_layout.rs`: public, pure geometry for header and building/panel bounds.
- `src/lib.rs`: exposes only that intentional geometry seam for integration tests. Most game code remains in the existing binary crate.
- `src/ui/common.rs`: runtime adapters to the shared layout plus remaining visual helpers.
- `src/ui/header.rs`, `src/ui/workspace_nav.rs`: global shell and navigation.
- `src/ui/building_view.rs`: cutaway sizing, clipping, selection, building Details control and visible scrolling.
- `src/ui/inspector_summary.rs`: on-demand building details, despite the legacy filename.
- `src/ui/apartment_panel.rs`, `hallway_panel.rs`, `application_panel.rs`, `ownership_panel.rs`: shared contextual bounds.
- `src/state/gameplay_views.rs`: selection rendering, close control, overlay input ownership, tutorial placement.
- `src/state/gameplay_actions.rs`: SelectBuilding action and other action routing.
- `src/ui/workspace_inbox.rs`: Tasks & requests entry point.
- `src/game.rs`: existing screenshot scenes plus the new `building_details` scene.
- `tests/screen_layout.rs`: five geometry regressions (full-width default, bounded side panels, bottom panels, wrapped header, no panel/world hit overlap).

## Next development steps

1. Visual state: completed in the continuation slice below (the original brief calls this Phase 2; the continuation numbered it step 1).
2. Tenant identity: extend `src/ui/resident_sprite.rs` with stable tenant-specific room dressing and a small ambient animation set. Keep decorative randomness separate from simulation RNG. Prefer existing identity/archetype/tenure data to new saved systems.
3. Time progression: build a brief monthly summary around `src/state/gameplay_turn.rs`, using actual transactions and events. Protect against repeated End Month input and define ordering with mandatory event choices, tutorial messages, campaign completion, and demo endings. Add seasonal lighting/background variation.
4. Polish: refine panel transitions, typography, spacing, colours, quiet notifications, environmental readability, touch targets, and reduced motion. Preserve consistent pixel rendering.

Continue in independently useful, validated commits. Preserve the distinction between completed layout work and future art/presentation work.

## Known follow-up details

- Visual-state simplification is now implemented: cutaway meters are removed, while exact condition remains in contextual panels.
- The old LayoutConfig fields for header, panel split, unit width/height and floor height remain for configuration compatibility, but the new responsive geometry no longer reads them. Unit gap is still read from configuration. Consolidate presentation tuning deliberately when expanding visual-state configuration; avoid adding another layout source.
- Existing secondary management screens have their own layouts. This slice changes their shared header and preserves routes, but does not fully redesign every screen for phone portrait.
- On a short bottom-panel workspace, the selected room can require Up/Down scrolling to bring back into view. Selection-aware camera positioning is a possible polish improvement.
- No new dynamic history or fixture-level maintenance model exists. Later visual progression must reflect actual upgrades and tenant changes.
- Legacy tests under `src/**/tests.rs` predate this work. New tests belong under `tests/`; do not expand legacy suites without first migrating the affected feature. Existing cutaway regression tests in `tests/ui/building_view.rs` retain their legacy include seam.

## Validation and capture workflow

Run `cargo fmt`, `cargo test`, and `cargo clippy --all-targets --all-features -- -D warnings`. After meaningful game changes, run `.\publish.ps1` with no parameters. It builds and deploys Windows and WebGL Preview; do not substitute a local server for this required step. Preserve root `catalog_thumbnail.png`.

Screenshots go directly in `docs/verification/`, replacing the same screen/state. The toolkit capture harness uses a temporary TSV manifest with `scene<TAB>absolute_png_path` rows. Set APARTMENT_CAPTURE_MANIFEST to that path, APARTMENT_CAPTURE_FRAMES (3 is enough for stationary layout), APARTMENT_WINDOW_WIDTH, and APARTMENT_WINDOW_HEIGHT, then run the built apartment executable from the project directory. Captures hide the native window automatically. The workspace debug executable is normally `../target/debug/apartment.exe`.

Useful scenes: showcase, unit_showcase, unit_scrolled, hallway_showcase, hallway_scrolled, building_details, applications_showcase, ownership_showcase, tutorial_showcase, tutorial_coach_showcase, inbox_showcase, tasks_showcase, tower_scrolled. The seeded scenes use deterministic state.

Compare desktop 1280 x 720, wide 1920 x 1080, short landscape, and portrait 390 x 844. Check all four contextual panel types, closing and deselection, vacant-unit applications, Tasks via Inbox, lobby repairs, menu/time controls, tutorial progression, and large-building scroll controls.

Before committing, follow AGENTS.md and rust_management/docs/COMMIT_STYLE.md, enforce the 800-physical-line Rust limit, inspect the working tree, and include all modified/untracked project files as required by the user's project instructions. Report the commit hash and validation result.

## First-slice validation record (2026-09-19)

- `cargo test -q`: 160 passed, 1 existing ignored balance-report test.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt` and `git diff --check`: passed.
- Native capture inspection: 1280 x 720, 960 x 540, 1920 x 1080, and 390 x 844. Reviewed building default, unit details, building details, hallway, Ownership, applications, tutorial coach, and Inbox entry to Tasks across those captures.
- `.\publish.ps1` without parameters: Windows and WebGL release builds and Preview file deployment succeeded. Project Roost tracking warned that http://127.0.0.1/project_roost/api/v1 was unavailable (connection refused on port 80). This did not stop packaging or Preview deployment.
- Preview destination: `\\wsl.localhost\Ubuntu\home\kalai\dev\games\apartment`.
- Screenshots are native rendered layout evidence, not a browser interaction or end-to-end campaign playtest. Browser release compilation was validated by publishing. The existing simulation, save, action, and geometry regression suites passed.

## Visual-state continuation (2026-09-19)

- Replaced red damage framing with deterministic stains, stepped cracks, scuffed flooring and debris. Wear progresses through the existing condition bands: below 25, below 45, below 75, and cared-for rooms at 75 or above. Repairing condition removes the corresponding layers; no new saved state or simulation RNG is involved.
- Shared-space condition now controls visible wear, litter, lamp appearance, clean flooring and plants. The entire lobby remains the touch target for the existing hallway panel and its accurately priced repair. Lamps are presentation of shared condition, not independently repairable fixtures.
- Existing kitchen, acoustic-panel and lighting upgrade layers render over the condition finish. Replaced the opaque-background decorative plant sprite with small pixel shapes.
- Removed apartment and lobby condition bars from the cutaway. Numerical condition remains available after selection. Updated leak coaching to describe cracks and worn floors.
- Apartment selection now exposes the existing repair action before the detailed stats, with the real amount, configured price and affordability check. Sold units retain their existing ownership panel. The original repair entry in the lower upgrades list remains available too.
- Added deterministic `condition_worn` and `condition_repaired` capture scenes for comparisons with identical tenants and upgrades. Refreshed building, apartment and hallway captures at desktop, wide, short landscape and portrait sizes.
- Base room textures still contain some baked-in wear and furniture; these overlays do not replace those assets. Tenant-specific dressing and ambient behavior remain the next step. No economy, ownership, campaign or save-format changes were made.
- Validation: 160 tests passed with the existing balance-report test ignored; formatting, strict all-target/all-feature Clippy, diff whitespace and the 800-line source limit passed. Native captures were inspected at 1280x720, 1920x1080, 960x540 and 390x844. These are rendering checks, not a browser interaction playtest. Default `publish.ps1` built and deployed Windows/WebGL Preview; Project Roost tracking again warned that the local port-80 service was unavailable.
