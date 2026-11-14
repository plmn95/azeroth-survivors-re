# Controls

**Azeroth Survivors** — third-person movement with independent mouse look.

| When | Key / Input | Action |
|---|---|---|
| Menu | Click a race, gender, class and zone | make your pick (**Randomize** rolls one) |
| Menu | **Enter** (or **Enter World**) | start the run; the world travels to the zone picked |
| Playing | **W / A / S / D** or **arrow keys** | move relative to the camera; the hero faces travel |
| Playing | **mouse movement** | look around the hero, with no mouse button held |
| Playing | hold **Alt / Option** | release the cursor for UI clicks, targeting or cursor aiming; let go to resume looking |
| Playing | **Space** | jump |
| Playing | **F** (from level 5) | your class's movement spell: directional spells follow the hero's facing; hold **Alt / Option** to aim at the cursor |
| Playing | **V** | enemy health bars on / off |
| Playing | **C** | character window: stats, abilities and talents |
| Playing | **Tab** (or click the icon at the bottom right) | cycle the focus the abilities pick targets by: nearest, lowest health, strongest |
| Playing | hold **Alt / Option** + **left click** on an enemy | target it: its frame shows beside yours, and the abilities hit it first while it is in range (a click on bare ground clears it) |
| Playing | (no key) | the quest tracker under the minimap lists the run's three quests: slay, gather the items a creature drops (walk over them), survive |
| Level up | (no key) | an ability at rank 8 with its matching talent at rank 3 offers an **Evolution** card; found recipes are kept in the spellbook shown at the bottom |
| Any | **G** | toggle the picture polish (`?grade=0` starts it off) |
| Playing | **mouse wheel** | zoom, from over the shoulder out to 70 yd |
| Playing | **Esc** | pause |
| Playing | — | abilities cast themselves (no aim) |
| Level up | **1 / 2 / 3** (or click a card) | pick an upgrade |
| Playing | **8 / 9 / 0** (or click a row) | take one of the three world buffs offered under the clock, at the start and every ten minutes; the first is taken after 15 s; the run does not pause |
| Menu | **Talents** button (under the zone list) | spend the character's talent points (one per level from 10), click a talent to add a rank; **Respec** gives them all back for free |
| Talents | **Esc** (or **Close**) | close the talents |
| Menu | **Normal / Heroic / Hardcore** (under the zone list) | enemy toughness and reward multiplier; Hardcore also drops no health gems; kept across runs |
| Paused | **Esc** | resume |
| Menu / Paused | **Options** button | sound volumes (drag a slider), mute, windowed or fullscreen; kept across restarts |
| Options | **Esc** (or **Close**) | close the options |
| Game over | **Enter** (or **Play again**) | back to the menu |
| Any | minimap **+ / −** / minimize | minimap zoom / collapse |
| Any | **F12** | screenshot, into the `--shots` folder or `Screenshots` in the state folder |
| Debug (`--debug`) | **F5** | hold the waves and the run clock / let them run again |
| Debug | **F6** | clear the field: every enemy dies, without loot |
| Debug | **F7** | god mode on / off (on at the start) |
| Debug | **F8** | the overlay (spawn rings, position, keys) on / off |
| Debug | **F9** | jump to the edge of the battleground the camera looks at |

The mouse looks around without turning a stationary hero. WASD and the arrows move relative
to the camera, with equal speed on diagonals; releasing the keys stops movement and keeps the
last facing. The closer, lower starting view is a local tuning choice; zoom and camera
collision still work.

Hold **Alt / Option** to use the cursor during a run. Pause, level-up choices, menus, the character
window, maps and options release it automatically; losing window focus also releases it.
In the menu the camera slowly orbits the preview hero; a left drag turns it and the wheel
zooms. Your race, gender, class and zone are remembered across reloads.
