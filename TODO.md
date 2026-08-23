# Second Story — Release Work

The remediation program and pre-release engineering blockers are complete.
This list covers the remaining work to turn the current playable build into a
release candidate. Work it in order.

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
