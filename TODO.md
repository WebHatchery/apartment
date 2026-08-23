# Second Story — Release Work

The remediation program is complete; this list covers the remaining work to
turn the current playable build into a release candidate. Work it in order.

## Release blockers

- [ ] Restore the strict lint gate on the current Rust toolchain. `cargo clippy
  --all-targets --all-features -- -D warnings` currently fails on needless
  lifetimes in `src/ui/ownership_panel.rs` and two manual even-number checks in
  `src/ui/resident_sprite.rs`. Apply the small idiomatic fixes, then pass fmt,
  clippy, and the full test suite with no warning allowances.

- [ ] Reconcile player-facing release metadata with the shipped game. The
  catalog page advertises three buildings while `assets/building_templates.json`
  contains six campaigns, and the README says the balance harness uses three
  strategies while it defines six. Rewrite the controls to lead with the
  visible tap/click actions; keyboard shortcuts remain supplementary.

- [ ] Establish a versioned save compatibility contract before the release.
  Add an explicit save-format version and fixtures for the pre-versioned save
  shape, then prove that `post_load` preserves valid progress while repairing
  legacy tenant addresses and blank building ids. Document how incompatible
  future saves are handled.

## Release candidate validation

- [ ] Rerun the 60-seed, six-campaign balance report after the release fixes.
  Confirm the Investor, Greedy, Neglect, rent-maximiser, condo, and portfolio
  policies still meet the balance targets; record the resulting table in the
  completion audit or release notes (the generated `balance_report.md` stays
  untracked).

- [ ] Perform a fresh-save browser QA pass using touch/click only at desktop
  and narrow viewports. Cover campaign selection, repairs, applications, rent,
  an event response, save/load, a win or loss, and the career summary. Refresh
  only the affected verification captures and retain the exact viewport sizes
  and outcomes with the release notes.

- [ ] Build and stage the release through the shared publishers: run
  `./publish.ps1`, inspect `./publish-itch.ps1 -DryRun` and `-Status`, and
  verify both Windows and HTML5 packages have the correct title, thumbnail,
  controls, and six-campaign description. Upload to itch.io only after the
  release owner approves the reviewed channel diff.
