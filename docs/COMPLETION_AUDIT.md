# Second Story — Remediation Completion Audit

Audited against `docs/REMEDIATION_PLAN.md` on 12 September 2026. The five
milestones remain complete after the release-candidate rerun. Evidence below
combines focused regression tests, the 60-seed representative balance harness,
fresh-save browser interaction checks, and both release publishers.

## Milestone 1 — Authoritative addresses and safe transactions

| Requirement | Evidence | Result |
|---|---|---|
| Building-scoped tenants, applications, rent, and relationships | `tenant_in_another_building_does_not_pay_active_building_rent`, `inactive_building_tenant_is_not_advanced`, both cross-building relationship tests, and stale-application coverage | Pass |
| Atomic condo sale and buyback | `sold_condos_leave_the_rental_roll`, `buyback_quote_does_not_change_ownership`, and gameplay action tests cover stale transactions; transaction code verifies funds before mutation and deliberately clears an occupied lease on sale | Pass |
| Sold units excluded from rental occupancy | `sold_condos_leave_the_rental_roll` and `test_vacancy_tracking` | Pass |
| Modal and pause focus | `overlays_reject_gameplay_actions_but_accept_event_responses`; browser Escape/pause check | Pass |
| Tutorial is finite and does not spam | `starter_resident_does_not_complete_acquisition_lesson`, `hint_only_emits_once_per_qualifying_month`, and `final_milestone_closes_after_mentor_messages_are_read` | Pass |

## Milestone 2 — One contract for every shipped feature

| Requirement | Evidence | Result |
|---|---|---|
| Building, Tenants, Finances, City, Inbox, and Tasks are visible routes | Shared workspace navigation plus browser traversal of every route | Pass |
| Mail, dialogue, missions, market, and portfolio have response routes | Inbox read controls, Tasks dialogue/mission actions, City market/portfolio controls, and narrative lifecycle tests | Pass |
| Upgrade definitions and effects are authoritative | `configured_amenities_update_authoritative_fields`, `test_soundproofing_effect`, `staff_factor_reflects_security_and_manager`, and stale-upgrade validation | Pass |
| Policies communicate and apply costs/benefits | Finance policy cards plus `open_house_charges_once_and_sets_duration`, utilities, insurance, receptionist, and janitor tests | Pass |
| Rent and mission effects use their named building | `rent_change_records_the_active_building`, both target-building mission tests, and named-building event-effect tests | Pass |
| Visible actions mutate state or explain why unavailable | Production `UiAction` route audit completed in milestone 2; unavailable ownership modes now state their external-governance rule instead of presenting an implementation placeholder | Pass |

## Milestone 3 — Representative balance and progression

Each entry is the mean of 60 deterministic runs using the live 36-month
rules. `V/A/B` means victory / all-tenants-left / bankruptcy terminal
outcomes. This is the 12 September 2026 release-candidate rerun.

| Campaign | Tier | Greedy score / cash | Investor score / cash / payback / V/A/B | Neglect score / condition / V/A/B | Rent max score / cash | Condo score / cash / sales | Portfolio score / cash / buildings |
|---|---|---:|---:|---:|---:|---:|---:|
| Sunset Apartments | Easy | 1312 / $80,270 | 1812 / $85,137 / 15.0 mo / 60/0/0 | 552 / 0 / 58/0/2 | 1438 / $141,957 | 1253 / $63,617 / 1.0 | 1441 / $74,849 / 0.0 |
| Riverside Court | Easy | 1594 / $146,088 | 2121 / $165,907 / 15.5 mo / 60/0/0 | 891 / 0 / 60/0/0 | 1812 / $252,190 | 1521 / $132,101 / 1.0 | 1807 / $130,234 / 0.2 |
| Blackwood Manor | Medium | 220 / $212,869 | 2671 / $340,124 / 14.9 mo / 59/0/1 | -7366 / 1 / 0/7/53 | 1208 / $366,661 | -146 / $149,272 / 0.8 | 2397 / $137,304 / 1.4 |
| The Foundry Lofts | Medium | 1222 / $159,261 | 2359 / $243,594 / 15.4 mo / 60/0/0 | -2041 / 7 / 33/0/27 | 1502 / $289,693 | 905 / $148,046 / 0.9 | 2061 / $130,902 / 0.8 |
| The Meridian | Hard | 1760 / $173,543 | 2474 / $238,637 / 15.1 mo / 60/0/0 | -5985 / 3 / 14/0/46 | 2185 / $340,985 | 1612 / $126,941 / 1.0 | 2145 / $114,036 / 0.9 |
| The Commons | Hard | -2285 / $33,984 | 1584 / $115,114 / 15.4 mo / 55/0/5 | -5732 / 20 / 5/0/55 | -982 / $108,310 | -2743 / $17,152 / 0.4 | -466 / $85,177 / 0.0 |

The investor strategy clears the 10% score target in every campaign and is
never behind Greedy in both score and cash. Investment payback stays within
14.9–15.5 months. Easy retains recovery room without twenty-times-starting-cash
inflation. Medium and Hard expose real failure profiles. Neglect sharply harms
condition, retention, score, and survival. Condo liquidity and rent maximization
can raise cash but do not dominate the care strategy's score.

The generated report also records occupancy, happiness, condition, application
volume, departures, investment, missions, grants, condo sales, and portfolio
expansion. `representative_balance_targets_hold` enforces the plan's target
bands. Investor still beats Greedy by at least 10% in every campaign, is ahead
in cash on every campaign, and pays back investment in 14.9–15.5 months.
Neglect remains materially worse on condition, retention, and terminal risk;
rent-maximisation, condo liquidity, and portfolio saving do not beat Investor
on score.

## Milestone 4 — Responsive management UI

Release-candidate browser QA used the freshly deployed full package with a new
save at 1280×720 and 800×600. The hosting shell preserved its 16:9 canvas at
1200×675 and 753×423.6 respectively. At both sizes the title screen exposed
all six campaign cards and gameplay retained the status bar, six labelled
workspace routes, cutaway, inspector, activity handle, and anchored tutorial
coach without overlap or unreachable controls.

At 1280×720 the pass selected Sunset Apartments, repaired the hallway (60% to
70%), listed a vacant apartment, accepted an application, raised another
vacant unit's rent from $650 to $700, resolved a developer-offer response,
saved at Month 6, reloaded, and restored that saved state. Continuing the same
run reached a Month 14 bankruptcy and displayed the career summary with the
distress outcome and final portfolio. At 800×600 a separate fresh run advanced
the tutorial, exposed and used the narrow hallway-upgrade drawer, and
traversed Building, Tenants, Finances, City, Inbox, and Tasks. A clean isolated
tab produced no browser warnings or errors after first render and interaction.

The existing committed captures already cover the same layout states and no UI
or artwork changed during this validation, so no unrelated captures were
regenerated. The six-unit and ten-unit layouts remain covered by
`ten_unit_cutaway_fits_short_workspace` and
`four_across_manor_units_remain_clickable_at_narrow_breakpoint`.

The root `catalog_thumbnail.png` remains a valid title-screen capture with the
current Second Story mark and all six campaign choices. It was not regenerated
because the title artwork and catalog composition did not change; milestone 4
changed responsive sizing and management presentation.

## Milestone 5 — Completion checks

- Seeded tenant archetype selection and upgrade menus now sort their configured
  inputs before applying RNG or strategy decisions. Three independent balance
  processes and three full default-parallel test processes produced stable
  results.
- The superseded mailbox renderer and its dead-code allowance were removed.
- No player-facing TODO, FIXME, HACK, “not yet implemented” placeholder, or
  obsolete mailbox compatibility path remains in `src`.
- `source_files_stay_under_the_limit` verifies the 800-line Rust source limit.
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: 153 unit tests passed, one report generator intentionally
  ignored, plus the two integration tests passed (155 passed in total).
- `cargo test balance_report -- --ignored --nocapture`: pass; all six campaigns,
  six strategies, and 60 seeds per strategy reproduced the table above.
- `publish.ps1`: pass on 12 September 2026; Windows and WebGL release builds
  packaged 70 registered assets and deployed the preview to
  `\\wsl.localhost\Ubuntu\home\kalai\dev\games\apartment` with the local catalog
  refreshed.
- `publish-itch.ps1 -DryRun`: pass; the standalone HTML5 stage contained the
  root launcher, demo WASM, packed assets, shared CSS, and six localized runtime
  bridges (11 files, 55.75 MiB), while the Windows channel staged the full
  `apartment_windows.zip` (29.21 MiB). No upload was performed.
- `publish-itch.ps1 -Status`: pass; Butler reported the configured `html5` and
  `windows` channels and their current remote builds.
- Static package inspection: the preview and itch launchers identify Second
  Story, expose the Tap/click, END MONTH, and MENU controls, and carry the
  six-campaign description; the preview catalog thumbnail is 1280×720, the
  itch stage contains `apartment.wasm`, `assets.zip`, and
  `apartment_windows.zip`, and the Windows archive contains `apartment.exe`
  and `assets.zip`.
