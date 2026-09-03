# Second Story — Remediation Completion Audit

Audited against `docs/REMEDIATION_PLAN.md` on 5 August 2026. The five
milestones are complete. Evidence below combines focused regression tests, the
60-seed representative balance harness, browser interaction checks, and the
normal WebHatchery publisher.

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

Each entry is the mean of 60 deterministic, 36-month runs. `V/A/B` means
victory / all-tenants-left / bankruptcy terminal outcomes.

| Campaign | Tier | Investor score vs Greedy | Investor cash | Payback | Investor V/A/B | Neglect V/A/B |
|---|---|---:|---:|---:|---:|---:|
| Sunset Apartments | Easy | 1812 vs 1312 (+38%) | $85,137 | 15.0 mo | 60/0/0 | 58/0/2 |
| Riverside Court | Easy | 2121 vs 1594 (+33%) | $165,907 | 15.5 mo | 60/0/0 | 60/0/0 |
| Blackwood Manor | Medium | 2671 vs 220 (+1114%) | $340,124 | 14.9 mo | 59/0/1 | 0/7/53 |
| The Foundry Lofts | Medium | 2359 vs 1222 (+93%) | $243,594 | 15.4 mo | 60/0/0 | 33/0/27 |
| The Meridian | Hard | 2474 vs 1760 (+41%) | $238,637 | 15.1 mo | 60/0/0 | 14/0/46 |
| The Commons | Hard | 1584 vs -2285 | $115,114 | 15.4 mo | 55/0/5 | 5/0/55 |

The investor strategy clears the 10% score target in every campaign and is
never behind Greedy in both score and cash. Investment payback stays within
14.9–15.5 months. Easy retains recovery room without twenty-times-starting-cash
inflation. Medium and Hard expose real failure profiles. Neglect sharply harms
condition, retention, score, and survival. Condo liquidity and rent maximization
can raise cash but do not dominate the care strategy's score.

The report also records occupancy, happiness, condition, application volume,
departures, investment, missions, grants, condo sales, and portfolio expansion.
`representative_balance_targets_hold` enforces the plan's target bands.

## Milestone 4 — Responsive management UI

Browser checks used 1280×720, 1024×768, and 800×600 windows. The hosting shell
preserved its 16:9 canvas at 1200×675, 977×550, and 753×424 respectively. At
each size the title screen exposed all six campaign cards and gameplay retained
the status bar, six labelled workspace routes, cutaway, useful inspector,
activity handle, and anchored tutorial coach without overlap or unreachable
controls.

Interaction checks traversed Building, Tenants, Finances, City, Inbox, and
Tasks; opened Hallway and Ownership inspectors; toggled activity history; and
verified Escape pause focus. The six-unit and ten-unit layouts are also covered
by `ten_unit_cutaway_fits_short_workspace` and
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
- `cargo test`: 123 unit tests passed, one report generator intentionally
  ignored, and the source-size test passed (124 passed in total).
- `cargo test balance_report -- --ignored --nocapture`: pass; all six campaigns,
  six strategies, and 60 seeds per strategy reproduced the table above.
- `publish.ps1`: pass; Windows and WebGL release builds packaged 67 assets and
  deployed the preview to `\\wsl.localhost\Ubuntu\home\kalai\dev\games\apartment` with the local
  catalog refreshed.
