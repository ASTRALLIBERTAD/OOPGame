# Archipelago Chronicles: Bahala Na Hindi Ko Pa Alam Title
## Game Design Document

**Genre:** 2D top-down procedural RPG
**Theme:** Philippine archipelago — governance as social metaphor
**Multiplayer:** Flexible solo + optional co-op (2–4p) + optional open server for 4 player plus

---

## Core Loop
Spawn in region → Do tasks/quests → Earn materials + piso → Craft or buy gear → Confront the Boss → Path choice:
- Path of Power: become the new Boss (do bad things)
- Path of Reform: rebuild the region (do good things)

**Economy rule:** Gathering raw materials + crafting is always cheaper than buying finished items.

---

## Entities (mobs) — governance as metaphor

### Neutral
- **Tagapamagitan** — roaming trader, profits from instability → middleman class
- **Mamamahayag** — journalist NPC, drops intel, befriendable → independent press
- **Estudyante** — recruitable student, drops pamphlets → youth shaped by who reaches them first

### Passive
- **Magsasaka** — farmer, produces food, flees hostile mobs, gives quests → working class
- **Pari / Pastor** — heals players, grants blessing buff, won't fight → institutional religion
- **Balikbayan** — one-time rare material trader → OFW diaspora

### Hostile
- **Snatcher** — steals piso, fast, low HP → petty crime from systemic neglect
- **Komisyon Goons** — checkpoint enforcers, work for Boss → bureaucratic rent-seeking
- **Troll Army** — spreads Confused debuff, travels in packs → disinformation networks
- **Puersa ng Orden** — aggressive in high Boss-influence zones → law as political instrument
- **Trapo Fixer** — bribes mobs, carries black funds → traditional corruption
- **Smuggler Vessel** — sea mob, politically protected → illegal trade networks

### Boss — The Buwaya
Crocodile in a barong tagalog. Three phases.
- Phase 1: Offers bribes (accept = short-term buff + long-term Indebted debuff)
- Phase 2: Calls in Troll Army + Puersa ng Orden
- Phase 3: True form — regens from nearby corruption tiles
- Weakness: corruption tiles cleansed by players stops regen
- Drops: Barong of Authority · Seal of Reform · Black Ledger
- Note: Not any one person — the system of extractive governance made flesh.

---

## 3 Worlds

| World | Biome clusters | Quest theme | Rare drops |
|---|---|---|---|
| Luzon | NCR urban, Cordillera, Ilocos, Bicol, CALABARZON | Urban poverty, land reform, heritage | Palayok armor, Igorot spear |
| Visayas | Western/Central/Eastern Visayas, Negros sugarcane | Labor rights, climate disaster, feudalism | Binakol shield, sinulog staff |
| Mindanao | Zamboanga, Davao, Caraga, BARMM, SOCCSKSARGEN | Indigenous rights, peace, resource extraction | Kris blade, malong cloak |

Biome purpose: different structures, quests, mob spawn tables, and material drops per biome.

---

## Procedural Terrain

Tile types:
- Rice paddy — farmable, drops palay
- Rainforest — dense resources, high mob rate
- Sea / river — boat travel, Smuggler Vessels
- Urban concrete — fast movement, low resources
- Mountain — slow movement, rare ores
- Corruption tile — hostile spawns, spreads over time
- Sacred ground — no hostile spawns, fast healing

Generation rules:
- Perlin noise seeds terrain height → biome type
- Boss influence (0–100) spreads corruption tiles from capital structures
- Clearing quests reduces influence radius
- Player homesteads persist between sessions
- Server season re-seeds terrain; player structures remain, mobs reset
- Bagyo (typhoon) events flood coastal tiles, boost sea mob spawns

---

## Multiplayer — Flexible by Design

**Core principle:** Everything works solo. Multiplayer only ever makes it better, never required.

### Session modes
1. **Solo / Offline** — full game, solo difficulty, governance choices yours alone, no PvP
2. **Party (2–4p)** — lan only, shared world + boss state, Bayanihan crafting, governance vote within party
3. **Open server** — public world, server events active, PvP opt-in, region-wide governance vote

### Boss scaling
- Solo: normal HP, cleanse tiles one at a time
- Duo (2p): +20% HP, two tiles simultaneously, shared respawn
- Squad (3p): +50% HP, goon wave added, simultaneous cleanse stuns boss
- Raid (4p): phase 3 unlocked, best drop quality, governance vote triggers

### Bayanihan crafting
- Solo: craft alone, normal quality
- 2–3p: split material contributions, +15% quality
- 4p: all types covered, +30% quality, Bayanihan item variant
- Cross-region: players from different worlds, fusion recipes unlocked

### Governance vote (post-boss)
- Solo: player decides alone
- Party: majority vote, ties to killing blow dealer
- Open server: 60s region-wide vote
- Offline: last cast vote still counts, no soft-locking

### Individual features (all work solo)
- Quests: solo by design, party auto-shares progress, each player keeps full drops
- OFW Trade Route: solo carries materials by boat; co-op adds escort/interception layer
- Troll Invasion: solo harder but possible; multiplayer contributes passively, no forced coordination
- Trapo PvP mode: fully opt-in, manual flag, off by default, solo players never pulled in
- Homestead: solo build, invite-only for co-op, shared homesteads get larger plots
- Region progress: per-player in solo, shared in multiplayer; solo progress carries forward
