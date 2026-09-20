# Second Story

Second Story is a cozy building-management game about caring for a neglected apartment block and the people who live there.

You are not a property tycoon. You are the custodian of a tired building with flickering lights, worn carpets, difficult choices, and tenants who need the place to work.

## Gameplay

- Inspect apartments and decide what to repair or upgrade.
- Review tenant applications and handle tenant needs.
- Adjust rent without destroying happiness or occupancy.
- Improve shared facilities and building condition.
- Manage money, time, requests, and long-term reputation.

## Goal

Survive 36 months while keeping the building financially stable and livable. Completing one building unlocks a harder property with new pressure.

## Controls

- Tap or click: select apartments and use labelled controls.
- End Month: tap the visible top-bar button to advance time. Space is an
  optional shortcut.
- Menu: tap the visible top-bar button to save or quit. Esc is an optional
  shortcut.

## Screen briefs and viewport contract

Second Story keeps the building and the player's current decision ahead of
management utilities. The shell uses a compact status row, a quiet destination
row, a world-first building view, and a retrievable activity footer.

| Screen | Current decision and primary action | Focus and supporting information | Deferred information and touch path |
| --- | --- | --- | --- |
| Building and selection | Decide what to inspect, repair, upgrade, or lease; tap the building, unit, or hallway, then use its labelled action. | The cutaway building is the dominant focus. A selected place shows identity, condition, price, affordability, and the immediate consequence. | Advanced stats, applications, ownership, and staff live behind the selected panel and its visible paging controls. Close returns to the building. |
| People | Compare residents and follow a resident need; select a resident or review applications. | The resident roster, name, unit, happiness, rent, and pending request count support the choice. | Aggregate statistics and application detail open from the selected list or Tasks route. |
| Finances | Decide which recurring operating commitments the building can carry this month. | Current cash, monthly income, outgoings, net result, and the price beside each policy action. | Lifetime totals, trends, and transaction detail are secondary ledger information. |
| City and market | Choose a neighborhood, compare a property, and review the purchase before committing. | The selected district or listing, price, affordability, occupancy, and purchase consequences lead the screen. | District statistics and the full portfolio list remain available after selection and pagination. |
| Inbox and Tasks | Read the selected letter or resolve a pending conversation, mission, or resident request. | One message or request at a time receives the reading and decision focus; the Tasks cue stays visible. | Other mail, mission pages, and request pages are reached with labelled paging and return controls. |
| Blocking and ending states | Resolve the required event/tutorial choice or understand the completed month/campaign result. | The active choice or concise result summary owns input and emphasis until it is dismissed or completed. | Save, quit, help, and history remain available through visible recovery controls when the state allows them. |

The hosted WebGL page keeps the canvas at the generated shell's 16:9 content
size (capped by its 1200 CSS-pixel game container) and synchronizes the drawing
buffer whenever that canvas or display scale changes. Full-screen play uses the
safe-area viewport. The supported review contract is a normal 1280×720 canvas,
a wide 1920×1080 canvas, a short 960×540 canvas, a minimum 640×480 landscape
canvas, and a 390×844 portrait canvas. These are canvas dimensions used by the
game layout and capture harness, rather than assumptions about a window frame.
Essential controls remain visible and touchable at the minimum landscape and
portrait sizes; smaller or narrower canvases are outside the support contract.

## Current Scope

Playable building progression with multiple properties, tenant systems, repairs, upgrades, missions, and month-by-month management.

## Demo builds

The `demo` Cargo feature produces the itch.io storefront edition. A demo run
lasts three monthly turns, representing about 90 in-world days, and ends with a
dedicated demo summary. Demo saves and campaign progress use their own storage
namespace, so they cannot overwrite the unrestricted game's saves.

```powershell
cargo build --release --features demo --target-dir target-demo
cargo build --release --features demo --target wasm32-unknown-unknown --target-dir target-demo
```

The itch.io publisher builds this demo automatically for the HTML5 channel and
uploads it in place of the unrestricted WebGL module. Pass `-SkipDemoBuild` only
when reusing an already validated module in `target-demo`.

## Save Compatibility

New saves use a versioned envelope (`save_format_version: 1`) around the game
state. Unwrapped saves from before this release remain supported: the loader
restores their non-persistent runtime fields, tenant building addresses, and a
missing current-building ID. A save from a newer format is rejected without
being overwritten, so the player can return to a compatible build instead of
losing progress.

## Balance Harness

`src/sim_harness.rs` plays the full 36 months headlessly under six strategies
(greedy, investor, neglect, rent maximiser, condo liquidity, and portfolio
saver) across many seeds and writes a comparison table:

```powershell
cargo test balance_report -- --ignored --nocapture   # writes balance_report.md
```

The report is generated output and is not tracked; regenerate it when tuning
the economy. A fast single-playthrough smoke test runs in CI as
`balance_harness_runs_without_panic`.

Open work is tracked in `TODO.md`.

## UI overhaul development

The first building-first layout slice is implemented. Read [the continuation guide](docs/UI_UX_CONTINUATION.md) for the remaining phases, implementation map, and validation workflow. The original [UI/UX brief](docs/UI_UX_OVERHAUL_BRIEF.md) is preserved alongside it.
