# Rust Game CI Failure Report

## Incident

The `Rust Game CI` run for commit `3603dbd` failed on 23 August 2026. The
`Check, test, and build WebGL` job stopped after 51 seconds, while the Windows
release job succeeded. GitHub identified the failing step as `Run Clippy`,
with exit code 101. The failure was therefore a quality-gate failure before
the WebGL build, not a game-runtime or deployment failure.

Run: <https://github.com/WebHatchery/apartment/actions/runs/32637741649>

## Root cause

The workflow installs the floating `stable` Rust toolchain and treats every
Clippy warning as an error with `-D warnings`. GitHub was running Rust 1.98,
where Clippy reports the `drain_collect` lint for this pattern:

```rust
let actions: Vec<UiAction> = self.pending_actions.drain(..).collect();
```

The code drained a vector and immediately rebuilt another vector of the same
type, creating an unnecessary allocation. Rust 1.96 accepted the code, which
explains why the older local check passed while CI failed after the floating
toolchain advanced.

## Fix

`src/state/gameplay_input.rs` now uses:

```rust
let actions = std::mem::take(&mut self.pending_actions);
```

This moves the pending actions out without allocating a replacement vector,
leaves an empty vector in the state, and preserves the existing action order
and modal-input behavior.

## Why other Rust games are affected

The shared CI workflow generator uses the same floating `stable` toolchain and
strict Clippy policy for 35 game repositories. A new Clippy lint can therefore
break several otherwise compiling games at once. A workspace scan found the
same literal `drain(..).collect()` pattern in `feast_frenzy` and
`dungeon_manager`; those repositories should receive the equivalent fix when
their CI runs report this lint.

## Verification

The exact standalone-checkout sequence now passes under Rust 1.98:

- `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features` — 145 tests: 144 passed and 1 ignored
- `cargo build --release --target wasm32-unknown-unknown`
- `publish.ps1` — Windows build, WebGL build, packaging, and preview deploy
