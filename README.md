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
