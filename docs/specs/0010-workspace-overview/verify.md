# Verify: Workspace overview · spec 0010 · updated 2026-10-29
_Steps derived from spec 0010 acceptance criteria. `/check verify` runs these; `/test` locks the durable ones._

## UI / manual
Build step 2 (thin thread). All passed on your screens 2026-10-29.
- [x] With windows on two or three screens, press Win+Tab → every screen goes dark grey with numbered tiles on top and live previews below; Task View does not open → AC-1, AC-3, AC-13
- [x] Click a window in the spread → you land on it and the overview closes on every screen → AC-6
- [x] Open it, click another workspace's tile → that screen switches and the indicator shows → AC-6
- [x] Open it, press Esc; open again, click empty background → focus goes back to the window you came from → AC-6, AC-8 (Esc)
- [x] With it open, press Win+Ctrl+Right → the overview closes first, then the switch runs → AC-12
- [x] With it open, press Alt+Tab → the overview closes and Windows' Alt+Tab opens; no overview popup is listed → AC-12

Build step 3 (mini windows, placement, cards, labels). Passed on your screens 2026-10-29.
- [x] Windows on several workspaces, one maximized, one minimized; Win+Tab → each tile shows its windows live at their real places, stacked as on screen; the minimized one at its restored place; the maximized one does not spill onto the next tile; numbers sit under the tiles; an empty tile shows nothing inside → AC-2
- [x] In the spread, a window from the left of the screen is on the left and one from the bottom is in a lower row → AC-3
- [x] Hover a spread window → its title shows under it, cut with … when long, and goes when you move away → AC-5
- [x] Click a tiny window inside another workspace's tile, or on another screen → you land on it → AC-6
- [ ] (optional check, not confirmed yet) `[overview] labels = "always"` → icon and title under every spread window; `"none"` → no text; `"sometimes"` → hover, plus one log line → AC-5, AC-14
- [ ] A window Windows draws nothing for shows as a dark card with its icon and title (rare: step 1 found DWM draws pdesk's hidden windows; check whenever one turns up) → AC-4

## Commands
- [x] `cargo test` → all pass (42 on 2026-10-29) → AC-2, AC-3, AC-14
- [x] `install.ps1` → log shows `shortcuts.overview=[Win+Tab]`, `overview.labels=hover` and `pdesk running` → AC-1, AC-14

## Value sourcing
- [x] Open on three screens → each overview covers its own whole screen (`ws::screens()`) → AC-1
- [x] Tile click → the outlined tile matched the workspace the screen showed (`ws::active`) → AC-2 (outline only)
- [x] A window on the right half of the second screen sits in the right half of that screen's tile, not offset by the screen's position (`ws::real_rect` mapped into the tile) → AC-2
- [x] Hidden windows stack in their tile as they did on screen: the front one is on top (`ws::windows()` order, reversed) → AC-2

## Acceptance-criteria coverage
- AC-1, AC-2, AC-3, AC-5, AC-6, AC-12, AC-13 covered by the step 2 and 3 checks · AC-4 card coded, not yet seen · AC-8 Esc only · AC-14 shortcut and `labels` only (`live`, `animation`: steps 6, 7)
- AC-7 (step 5), AC-8, AC-9 (step 4), AC-10 (step 6), AC-11 (step 7): not built yet
