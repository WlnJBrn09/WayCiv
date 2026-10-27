# WayCiv

A Civilization-style 4X game written in Rust, with egui. WayCiv follows Civ
VI's turn flow with history's leaders and empires, and its own content and
simpler rules. It's standalone: no other game needs to be installed.

All content (33 civilizations, 35 leaders, about 110 units, 38 wonders, 69
techs, 10 governments, 94 policies, 176 great people, and more) is WayCiv's own
data in `crates/wayciv-data/data/*.toml`, compiled into the game. Real leaders,
nations, and places are kept; every ability, stat, description, agenda, and
line of dialogue is written for WayCiv. There are **no civics**: governments,
policies, and other former civic unlocks arrive when the calendar reaches
their year. `cargo run -p wayciv-core --example catalog` prints the whole
catalog as Markdown.

```bash
cargo run --release -p wayciv
```

## How a game flows

1. **Create Game**: pick a leader from the portrait grid. Their leader ability,
   civ ability, and agenda are shown. Choose the map (Earth or Random), map size,
   rivals, city-states, and seed.
2. **Turn 1**: found your capital with the Settler (`B`, or the Found City button).
3. Each turn, as in Civ VI, the End Turn button shows what's blocking it: "Choose
   Research", "Choose Production", "Unit Needs Orders", "Choose Pantheon",
   "Choose Policy Card", "Choose Dedication". Clicking it takes you to the
   blocker. `Enter` presses it.
4. AI civilizations and city-states take their turns. Then cities grow and build,
   and science, culture, faith, and gold accumulate.
5. Meeting leaders, wars, denouncements, demands for tribute, friendship and
   alliance offers, calls to arms, and peace offers open a leader screen with
   their portrait and their own lines.
6. Historic moments earn era score; each new era brings a Golden, Normal,
   Dark, or Heroic Age and a dedication. Completing a wonder shows it full
   screen.
7. Win by Science (space projects), Culture (tourism), Domination (all original
   capitals), Religion, or Score at the turn limit. Everyone is warned when a
   civilization is 50%, 75%, and 90% of the way to a victory. The end screen
   graphs every civilization's score, science, culture, military, and cities
   over the game.

Controls: left-click selects, right-click moves or attacks, drag or the arrow
keys scroll, the mouse wheel zooms. The minimap (bottom right) recenters the view.

| Key | Action | Key | Action |
|---|---|---|---|
| `N` | Notifications | `L` | Leaders and relationships |
| `S` | Research (tech tree) | `A` | AI tree (once unlocked) |
| `R` | Religion | | |
| `G` | Government | `P` | Great people |
| `E` | City-states | `V` | Victory progress |
| `T` | Eras and timeline | | |
| `K` | Map key | `Enter` | Next turn, or the next thing that needs a decision |
| `.` | Next unit | `Space` | Skip unit |
| `F` | Fortify | `Z` | Sleep |
| `B` | Found city | `Esc` | Cancel, then close windows, then deselect |

Pressing a window's key again closes it. Notifications stay off screen until
you press `N`; new ones appear briefly as toasts at the right edge.

The layout follows Civ VI on the Nintendo Switch: research (and AI) trackers
top left, leaders top right, the unit or city panel at the bottom left and
right, and the big Next Turn button with the minimap at the bottom right.
The menu buttons at the top right show their hotkeys.

Every menu works the same way. A teal strip under the title says what to do
next. Choices are cards: click one to pick it, a gold frame marks what you
have, and dimmed cards can't be picked yet (hover for why). A menu closes once
its decision is made: picking research, a pantheon, founding a
religion, confirming policies, sending your last envoy, or choosing what a
city builds. Policies are Civ VI-style cards (red military, yellow economic,
green diplomatic, purple great person) that you click into slots, then
confirm. A Great Prophet's Found Religion button opens the founding screen:
pick a name, a founder belief and a follower belief.

## The map

- **Earth** (the default) is the real world: continents, mountain ranges,
  rivers, and biomes come from NOAA's ETOPO1 elevation and ocean depth and
  WorldClim 2.1 temperature and rainfall (`tools/earth/build_earth.py`). Rivers
  follow real drainage. Every leader starts at their historical capital
  (Reagan at Washington, Victoria at London, Qin at Xi'an...), city-states sit
  at their real cities, and natural wonders are where they are on Earth (Everest,
  Kilimanjaro, the Great Barrier Reef...). Sizes: Small 104x72, Standard
  128x88, Giant 160x110 (the default).
- **Random** generates a fractal world with the same biomes.
- Hex outlines are always drawn.
- Biomes: grassland, plains, desert, tundra, snow, coast, sea, and deep sea,
  with forest, taiga, rainforest, marsh, floodplains, desert floodplains, oasis,
  dunes, glacier, and sea ice on top, plus hills, mountains, cliffs, rivers, and
  natural wonders. Taiga is like forest and grows in the cold north. Dunes cost
  an extra move. Glaciers (Greenland, Antarctica) are impassable. Cliffs line
  steep coasts and give +3 defense. Deep sea is open ocean, crossed with Cartography.
- Units and resources are drawn with their icons when you zoom in.
- Each civilization's territory is tinted with its color and outlined with a thick
  border in its primary and secondary colors. Cities have banners in the owner's colors.
- "Map key" (bottom left) shows the legend. Hovering a tile shows its biome,
  features, resource, yields, and owner.

## What's simplified

- **Combat** uses Civ VI's damage formula and wall rules, which were recovered
  from the game's binary in `~/Projects/civ6-decompile`. Promotions are simplified
  to levels (+3 strength each).
- **Policies, governments, beliefs, leader and civ abilities, and wonders** each
  list their effects in the data (`wayciv_data::Effect`): city and capital
  yields, district, building and improvement yields, production bonuses, great
  person points, housing, amenities, combat strength, trade, and more. Every
  effect applies, and descriptions are generated from the effects.
- **Governments and policies unlock by calendar year** instead of civics. When
  a year brings new ones, everyone gets a free policy change and an envoy. The
  Government screen shows what unlocks next and on which turn.
- **Policy cards** (`policy_cards.rs`): you can only slot the cards you have
  kept. Culture fills an inspiration meter; when it is full, three unlocked
  cards are offered (newer ones more often) and you keep one. A card leaves
  once you keep the card that supersedes it. The first offer comes on turn 1.
- **Eras and ages** (`eras.rs`): the world's era follows the calendar.
  Historic moments earn era score: founding cities, meeting civilizations,
  discovering natural wonders, wonders, religions, great people, clearing
  camps, capturing cities, winning wars, cities growing to 10/15/20, the first
  district of each kind, new governments, and firsts in the world. When the
  next era begins, your score sets your age: Dark, Normal, Golden, or Heroic (a
  Golden Age right after a Dark one). Thresholds scale with the era's length
  and your empire's size. Each age comes with a dedication (two in a Heroic
  Age): in a Golden or Heroic Age its bonus applies (+15% Science, +15%
  Culture, district and wonder production, +20% Gold, +5 combat strength, or
  +20% Faith); otherwise it earns +1 era score for its kind of moment. Golden
  Ages give +3 Loyalty per city; Dark Ages take 3.
- **Rivals** (`rivalry.rs`): every leader has a personality in
  `leaders.toml` (`ai`: aggression 0-3, a focus, and the rule their agenda is
  scored by). Agendas really change opinions (Gorgo admires wars, Qin
  wonders, Philip shared faith, Lady Six Sky distance...). Surprise wars,
  captured cities, refused demands, broken alliances, and settling too close
  leave grievances that fade in peacetime. AIs plan wars: they denounce the
  target, mass troops on the border (you get a warning), then declare war and
  invite their friends. Strong AIs demand tribute, and you can demand it from
  them. Friends of long standing can form alliances (shared maps, and allies
  join each other's defensive wars when called to arms). Peace pays
  reparations to the side ahead on war score, and a truce follows. The AI
  spends its Gold and picks governments, policies, and dedications by focus.
- **Tribal villages** (`villages.rs`) dot the unclaimed land at the start,
  glowing gold. The first explorer to enter one gets a gift: Gold, Faith, a
  Eureka, a citizen, a map of the region, a Builder or Scout, or a level.
- **The wonder race**: you hear when a civilization starts a wonder, the city
  panel shows rivals building the same one, and a wonder lost to a rival
  refunds its progress as Gold.
- **Great people** use one simplified effect per class, plus WayCiv's modern
  great people: Elon Musk (minor boosts to AI, rockets, and solar), Gwynne
  Shotwell (a big boost to a space race project), Palmer Luckey (a flight of
  Anduril Strike Drones), and Ilya Sutskever (a big AI breakthrough) as
  engineers and scientists; Yanni and NF as musicians; and Michael Crichton,
  Iain M. Banks, and Douglas Adams as writers.
- **Religion** uses pantheons, founding at a Holy Site with a Great Prophet,
  missionaries, and passive pressure. There is no theological combat.
- **City-states** give envoy and suzerain bonuses by category. There are no
  quests, and no one can declare war on a city-state.
- **Loyalty**: every city within 9 tiles pulls on a city, harder when bigger and
  closer. Your cities hold it; foreign cities pull it away. Capitals (+5),
  garrisons (+2), Monuments (+1), and happy citizens add loyalty; occupation (-3)
  and unhappiness take it away. A city at 0 loyalty joins the civilization
  pulling on it hardest. The city panel shows the breakdown, and banners show a
  loyalty bar when it drops.
- **Civilians** (settlers, builders, traders, great people...) have no health.
  One attack from a combat unit kills them.
- **Artificial Intelligence** (a new Information Era tech after
  Telecommunications) unlocks **Datacenter** districts and the **AI tree**, a
  third research tree researched with Compute instead of Science. Datacenters
  make Compute: +6 each, +2 per adjacent district, +3 next to a river or the
  coast, more with a GPU Cluster or Solar Farm. The tree gives large bonuses:
  Science from Machine Learning, Deep Learning, Large Language Models, AI
  Research Agents (free Eurekas), Automated Science (cheaper space race), and
  Superintelligence; combat strength from Computer Vision; Production from
  Robotic Automation; and Anduril Strike Drones from Autonomous Systems.
- **Anduril Strike Drones** come off the line three at a time. They fly over
  rough terrain (every land tile costs one move), ignore zones of control, and
  strike from 3 tiles away.
- **Buying**: right-click anything in a city's production list to buy it with
  Gold. The current build keeps going.
- **Trade**: Traders (from 2500 BC) start routes from your cities to
  cities within 15 tiles. Domestic routes bring Food and Production. International
  routes bring Gold, more for destinations with more districts. Routes last 30 turns,
  then the Trader comes home. Capacity comes from the Palace, Commercial Hubs,
  Harbors, and effects (Colossus, Merchant Republic, some policies...). Each route lays
  a road, and roads make any land tile cost one move.
- **Aircraft** are built at and based on Aerodromes (4 slots). They strike anything
  within their operation range and rebase between Aerodromes. Anti-air units next
  to the target and enemy fighters covering it fire back. Losing a city loses the
  aircraft based there. In WayCiv, the Aerodrome doesn't count toward the
  district limit.
- **Barbarians** spawn guarded camps in unclaimed fog, at least 5 tiles from any
  city. Camps send out era-appropriate raiders that pillage, kill civilians, and
  attack weak targets. They never take cities. Clearing a camp pays 50 Gold.
- Not included: espionage, theological combat, and city-state quests.

## WayCiv's changes to the roster

**Ronald Reagan** leads America in place of Teddy Roosevelt. **Gandhi is not
included**, so India isn't in the game.

- **Peace Through Strength** (leader ability)
  - **Defense buildup**: +15% Production toward military units. +5 Combat
    Strength for units fighting inside your territory.
  - **Reaganomics**: +1 Gold in every city for each Economic policy slotted.
    Commercial Hubs give +2 Production.
  - **The Great Communicator**: +2 Culture in the Capital. Declaring Friendship
    grants an Envoy and 50 Gold.
- **Agenda: Cold Warrior.** Likes civilizations running Democracy. Dislikes
  civilizations running Communism or Fascism.

## Leader portraits and icons

The portraits in `assets/leaders/` are original paintings generated with
Codex image generation, one per leader. A leader without a portrait shows a
monogram in their civilization's colors.

The icons in `assets/icons/` (resources, units, techs, AI tree items,
beliefs, and the trees themselves) are generated the same way:

```bash
cargo run --release -p wayciv-core --example icon_list > tools/icons/icons.tsv
tools/icons/generate.sh tools/icons/icons.tsv 4   # skips icons that exist
```

Anything without an icon falls back to rendered art (below) or a symbol.

## Map art

Everything on the map is original art rendered from scripts, so it can be
rebuilt or changed:

| Art | Assets | Built by |
|---|---|---|
| Terrain: biome textures, hills and mountains, features, natural wonders | `assets/terrain/` | `python tools/terrain/build.py` (numpy, scipy, Pillow) |
| Units: 3D models with team-color masks, plus portraits | `assets/units/` | `tools/units/build_all.sh <python>` (Blender, Pillow, numpy) |
| Districts, wonders, and city centers (by era, size, and walls) | `assets/buildings/` | `tools/buildings/build.sh` (Blender) |
| Resources, improvements, roads, barbarian camps | `assets/resources/` | `tools/resources/build.sh` (Blender) |

The Blender scripts run headless (`blender -b`) and render low-poly models
from a 3/4 view. The code that draws each kind lives in
`crates/wayciv/src/render/`; when an image is missing it falls back to simple
vector drawings.

## Pacing

Pick a game length on the setup screen: 100, 250, 500, 750, or 1000 turns, or
Infinite. Shorter games run faster: research, production, growth, border
growth, great people, and faith costs all scale with the length, so every
length reaches the late eras near its last turn. 500 turns is the standard
pace. Infinite games have no turn limit and run at 500-turn speed.

The calendar runs from 4000 BC to about 2150 AD on the last turn, stretched to
fit the length. It follows the typical pace of research (in all-AI games the
average civilization reaches the Modern era around 1940), so year-gated
governments and policies arrive alongside the techs of their time.

The base rules (for a 300-turn pace) keep the game moving through all eras:
- Techs cost 60% of their listed Science.
- City centers add +2 Food and +2 Production.
- Each citizen adds Science and Culture that grows with the era, plus 1 Gold per
  2 citizens.
- Amenity shortfalls are milder than in Civ VI.

## Crates

| Crate | Purpose |
| --- | --- |
| `wayciv-data` | WayCiv's content as TOML (`data/`), its types, effects, and validation. |
| `wayciv-core` | The game: map generation, cities, units, combat, research, religion, diplomacy, AI, victories. No UI. |
| `wayciv` | The egui app. |

```bash
cargo test                                                  # engine + UI flow tests
cargo run --release -p wayciv-core --example autoplay -- 300 7   # all-AI game
cargo run -p wayciv-core --example catalog > WayCiv-Content.md   # content catalog
```

## License

WayCiv's code, content, and assets are under the [MIT License](LICENSE). WayCiv
is inspired by Civilization VI (Firaxis Games and 2K) but contains none of its
data, text, or art, and doesn't need it to run. The Earth map is
derived from NOAA ETOPO1 (public domain) and WorldClim 2.1 climate data.
