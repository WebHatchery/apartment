# Second Story UI/UX Overhaul Brief

## Goal

Redesign the main building screen so the game feels like a cozy, lived-in apartment-management game rather than a property-management dashboard.

The building itself should become the main interface.

The player should be able to understand much of the building state visually, without relying on large permanent information panels.

Do not change the underlying game systems unless needed to support presentation.

---

# Core Design Principle

## World first, UI second

The building should occupy most of the screen.

The current interface gives too much space to:
- large navigation bars
- permanent inspector panels
- oversized buttons
- empty background
- duplicated status information

The player should spend most of their time looking at:
- apartments
- tenants
- shared spaces
- visible building problems
- visible upgrades

UI should appear contextually around the building instead of enclosing it.

---

# Target Screen Balance

Aim approximately for:

- 65–75% building/world
- 15–20% contextual information
- 10–15% persistent navigation/status

The apartment building should be substantially larger than it is currently.

Reduce empty sky/background space.

Do not make the player feel like they are viewing the building through a small viewport inside a dashboard.

---

# 1. Simplify the Persistent UI

Keep a compact persistent top bar.

Suggested structure:

Second Story | Building | People | Finances | City | Inbox

Right side:

Money | Month | Occupancy | End Month | Menu

Avoid multiple stacked navigation rows where possible.

Remove or collapse:
- Lobby / Applications / Ownership as permanent large tabs
- permanent Quick Actions
- permanent Activity panel unless there is meaningful activity to show

Secondary screens should open when required rather than permanently occupying space.

---

# 2. Remove the Permanent Building Inspector

The current right-side Building Inspector uses too much screen space for too little information.

Replace it with contextual panels.

Default state:
- building fills most of the available width
- only compact overall stats are visible

Example:

$10,000 | 1/6 occupied | 🙂 70% | 🔧 53%

When the player selects something, show a contextual panel.

Examples:

## Apartment selected

Show:
- tenant or vacancy
- rent
- condition
- happiness
- outstanding issues
- repair/upgrade actions
- lease information

## Hallway selected

Show:
- shared-space condition
- known problems
- repair options
- effects on tenants

## Building selected

Show:
- overall condition
- occupancy
- monthly income/expenses
- major building-wide issues

The contextual panel can slide in from the right and disappear when nothing is selected.

---

# 3. Make Apartments Visually Communicate State

Do not rely primarily on bars and text.

Apartment visuals should reflect:

- condition
- occupancy
- tenant personality
- repairs
- upgrades
- long-term history

Examples:

Poor condition:
- stains
- cracked walls
- broken fixtures
- damaged flooring
- exposed wiring
- worn furniture

Improved condition:
- fresh paint
- repaired flooring
- better lighting
- nicer fixtures
- improved furniture

Occupied apartments should gradually become visually distinct.

A tenant may introduce:
- plants
- books
- clutter
- posters
- decorations
- exercise equipment
- pets
- hobby items
- different furniture arrangements

The apartment should become part of the tenant's identity.

---

# 4. Make Tenants Visible in the Building

Tenants should not exist only as portraits and stats.

Add lightweight ambient behaviour where practical.

Examples:

- walking through the lobby
- entering/leaving apartments
- carrying groceries
- sitting in shared spaces
- talking to another tenant
- checking mail
- waiting near a broken fixture
- moving boxes during move-in/out
- decorating their room
- watering plants

These animations can be mostly cosmetic.

The goal is to make the building feel inhabited without adding complex simulation requirements.

---

# 5. Improve Shared Spaces

Shared areas should be visually important.

The lobby and hallway should act as a visible representation of overall building health.

At the beginning:

- poor lighting
- worn carpet
- damaged walls
- broken bench
- old mailboxes
- water stains
- sparse decoration

As the building improves:

- repaired lighting
- cleaner surfaces
- better flooring
- plants
- seating
- improved mailboxes
- artwork
- noticeboards
- decorations

Players should be able to visually compare Month 1 and Month 30 and immediately see how much the building has changed.

---

# 6. Show Problems In The World

Where possible, convert abstract warnings into visible environmental issues.

Instead of only:

"Hallway needs attention: 60% condition"

Show:

- flickering light
- stained wall
- damaged carpet
- leaking pipe
- bucket catching water
- broken bench

Clicking the visible issue should open its information/action panel.

Example:

Hallway Light
Condition: Poor

Sarah has complained about the lighting.

Repair: $180

Avoid excessive floating icons.

Use environmental visuals first, subtle icons second.

---

# 7. Make The Building The Main Navigation Surface

Whenever possible, let players interact directly with the building.

Examples:

Click apartment → apartment/tenant panel

Click vacant apartment → applications / advertise / renovate

Click lobby → shared space options

Click visible maintenance problem → repair details

Click tenant → tenant information

Avoid forcing players through multiple menus for actions that naturally belong to something visible on screen.

---

# 8. Improve Apartment Readability

Each apartment should clearly show at a glance:

- unit number
- occupied/vacant
- basic tenant identity if occupied
- major active problem if one exists

Avoid placing multiple tiny bars and text labels inside every apartment.

Use colour, environment, and character presence instead.

Keep text overlays minimal.

---

# 9. End Month Should Feel Important

The game revolves around 36 monthly turns.

End Month should feel like advancing time, not clicking a generic button.

After pressing End Month, show a short transition or summary.

Example:

## October → November

Rent collected: +$2,400

Repairs: -$650

Sarah renewed her lease.

3A received two applications.

The hallway leak worsened.

Then return to the building and visually update anything that changed.

Keep this sequence concise.

It should take seconds, not become a long report screen.

---

# 10. Show Passage Of Time

Use the 36-month campaign visually.

Add seasonal/environmental variation where practical.

Examples:

Spring:
- brighter lighting
- rain
- plants

Summer:
- warm sunlight
- open windows
- fans

Autumn:
- orange evening light
- leaves outside

Winter:
- darker exterior
- rain
- condensation
- heaters

These do not need deep gameplay mechanics.

They exist primarily to make the building feel like the player has spent years caring for it.

---

# 11. Reduce Dashboard Styling

The current design contains too many:

- dark rectangles
- border boxes
- full-width buttons
- tiny labels
- spreadsheet-like status areas

Move toward a quieter interface.

Prefer:

- contextual cards
- smaller controls
- tooltips
- direct interaction
- subtle overlays
- visual state within the building

The UI should support the building rather than compete with it.

---

# 12. Preserve The Pixel-Art Direction

Do not abandon pixel art.

Instead, improve visual cohesion.

Target:

- warm interior lighting
- cooler exterior/background colours
- more detailed apartment interiors
- stronger depth and shadows
- readable silhouettes
- distinct tenant rooms
- visible wear and repair states
- less empty space

Avoid making the graphics overly clean.

The building should feel slightly imperfect and lived in.

---

# 13. Strengthen Character Presence

Character portraits can remain for:

- dialogue
- applications
- tenant requests
- major events

But characters should also exist spatially within the building.

Do not make every tenant interaction a modal dialogue.

Use short messages and environmental storytelling where possible.

Example:

Sarah:
"The heater is making that noise again."

This could appear as a compact message at the bottom while the player continues looking at the building.

---

# 14. Improve The Opening Tutorial Presentation

The Uncle Artie introduction currently blocks a large portion of the building.

Keep dialogue overlays smaller.

Allow the building to remain visible during tutorials.

Use highlights to point toward relevant areas.

Example:

Uncle Artie:
"Start with the hallway. People forgive an ugly apartment before they forgive a dark staircase."

Highlight the hallway.

Avoid large tutorial panels unless necessary.

---

# 15. Empty Space And Composition

The current screen has too much unused sky around the building.

Increase building scale.

The building should feel anchored to the bottom/middle of the screen.

Do not centre a small building inside a large decorative background.

Desktop layouts should make good use of widescreen monitors.

On wide screens:
- building expands
- contextual panel remains reasonably narrow

Do not simply stretch UI panels horizontally.

---

# 16. Visual Progression Is Critical

Player investment should create visible before/after change.

A successful campaign should transform:

Neglected building
→ functional building
→ comfortable building
→ genuinely pleasant home

This should happen through:

- repaired architecture
- upgraded shared spaces
- tenant decorations
- better lighting
- cleaner surfaces
- more activity
- visual signs of community

The player should feel ownership because they remember how bad the building used to look.

---

# 17. Avoid Adding New Feature Bloat

This pass is primarily UX and presentation.

Do not respond to these problems by adding many new game systems.

Do not add:
- complex relationship simulation
- dozens of new tenant stats
- unnecessary currencies
- excessive random events
- large new management screens

Focus on making the systems already present feel more tangible.

---

# Suggested Main Screen Structure

Target something conceptually similar to:

┌──────────────────────────────────────────────────────┐
│ Second Story  Building People Finances City     $10k │
│                                    Month 4 [End Month]│
├──────────────────────────────────────────────────────┤
│                                                      │
│                   BUILDING                           │
│                                                      │
│      ┌──────────┬──────────┐                        │
│      │    3A    │    3B    │                        │
│      │  Vacant  │  Vacant  │                        │
│      ├──────────┼──────────┤        Context panel   │
│      │    2A    │    2B    │        when selected  │
│      │  Vacant  │  Sarah   │                        │
│      ├──────────┼──────────┤                        │
│      │    1A    │    1B    │                        │
│      ├──────────┴──────────┤                        │
│      │       Lobby         │                        │
│      └─────────────────────┘                        │
│                                                      │
├──────────────────────────────────────────────────────┤
│ Sarah: "The heater is making that noise again."      │
└──────────────────────────────────────────────────────┘

The exact layout can change.

The important part is the hierarchy:

1. Building
2. Current situation
3. Contextual actions
4. Global management screens

Not:

1. Navigation
2. Dashboard panels
3. Statistics
4. Building

---

# Implementation Priority

Implement this in stages.

## Phase 1: Layout

- enlarge the building
- reduce background space
- remove permanent inspector
- simplify top navigation
- create contextual side panel
- reduce oversized buttons

## Phase 2: Visual State

- apartment condition variants
- shared-space condition variants
- visible maintenance problems
- basic occupied/vacant differences

## Phase 3: Tenant Identity

- tenant-specific room dressing
- simple tenant presence/animation
- move-in and move-out visuals

## Phase 4: Time Progression

- monthly summary
- seasonal background/lighting changes
- visible month-to-month changes

## Phase 5: Polish

- transitions
- hover states
- compact notifications
- animation timing
- typography
- spacing
- final colour pass

---

# Success Criteria

The redesign succeeds if:

- the building dominates the screen
- the player understands apartment state without reading many numbers
- occupied apartments feel inhabited
- building problems can be seen
- improvements visibly change the property
- tenants feel like residents rather than database entries
- the UI contains fewer permanent panels
- advancing a month feels meaningful
- Month 30 looks substantially different from Month 1
- the game feels cozy and human rather than corporate

The most important outcome is:

The player should feel that they are caring for a place and the people inside it, not operating property-management software.