# Mob Spawn System

A data-driven spawner. It reads the player's current **region**, **subzone**, and
**in-game time-of-day**, then spawns the right mob types in the right group sizes
at the right frequency. All spawn rules live in **data** (a JSON table + one
config Resource) — there is no per-region/per-mob branching in code, so designers
retune everything without touching GDScript.

## Files

| File | What it is |
|---|---|
| `spawn_rules.json` | The spawn table: `region → subzone → mob → { density, time, group_min, group_max, location }`. The bulk tunable data. |
| `SpawnConfig.gd` / `spawn_config.tres` | The **single config block**: region multipliers, density→weight map, respawn windows, danger zones, time-of-day cycle, difficulty/elite hook flags. Edit the `.tres` in the Inspector. |
| `MobTypes.gd` | The mob **enum** + registry: maps each mob key → scene path, class (Passive/Neutral/Hostile), display name. |
| `SpawnManager.gd` / `SpawnManager.tscn` | The spawner node. Resolves context → candidates → rolls → spawns, with respawn cooldowns and a population cap. |

## How a spawn is decided

Every `evaluation_interval_seconds` (default 5s), for the current region+subzone:

```
for each mob row in spawn_rules[region][subzone]:
    if row.time window does NOT include the current time band:  skip
    chance = base_spawn_scalar
           × region_multiplier[region]        # baseline encounter rate
           × density_weight[row.density]       # subzone density weight
           × danger_multiplier (if applicable) # danger-zone hook
           × event_multiplier  (if applicable) # event hook
    chance = clamp(chance, 0, 1)
    if mob is on respawn cooldown:  skip
    if randf() < chance:
        group = randi_range(row.group_min, row.group_max)
        spawn `group` mobs near the player
        start this row's respawn cooldown
```

`resolve_candidates(region, subzone, band)` does the pure part (no rolling, no
side effects) and is the easiest entry point for testing/inspection.

### Tunable mappings (in `spawn_config.tres`)

- **Region → multiplier:** NCR 1.0, CALABARZON 1.0, Ilocos 0.6, Bicol 0.6, CAR 0.35.
- **Density → weight:** `very_low` 0.1, `low` 0.3, `med` 0.6, `high` 1.0.
- **Respawn windows:** default 300–600s; high-density rows use the fast 120–240s window.
- Effective chance ≈ `region_multiplier × density_weight` (then the hooks above).

### Allowed enum values (validated at load)

- **density:** `very_low`, `low`, `med`, `high`
- **time:** `day`, `evening`, `night`, `day_evening`, `evening_night`, `24h`
  (`24h` always passes; NCR Snatcher/Thug use `evening_night`)

## Wiring it in

Registered as an autoload in `project.godot`:

```ini
[autoload]
SpawnManager="*res://Mobs/Spawning/SpawnManager.tscn"
```

(The script intentionally has **no `class_name`** — the autoload already provides
the global `SpawnManager`, and a same-named `class_name` would conflict.)

It self-discovers the player via the `player` group and stays idle while no
player exists (so it won't spawn in menus). From your region-streaming / world
code, feed it context:

```gdscript
SpawnManager.set_context("NCR", "periphery_slum")    # region + subzone
SpawnManager.set_time_of_day(SpawnManager.Band.NIGHT) # optional override; -1 = use clock
```

If you don't override the time, the built-in clock cycles over
`day_length_seconds` (default 1200s) using `evening_start`/`night_start`.

Spawned mobs join the `spawned_mob` group and carry metadata
(`mob_key`, `spawn_region`, `spawn_subzone`, `spawn_location`). The
`mob_spawned(mob_key, instance)` and `spawn_cycle_evaluated(...)` signals are
available for HUD/debug.

### Lifecycle & cleanup

Because it's an autoload, spawned mobs are **parented to the manager** — they
live under `/root` and survive scene changes. The manager owns them and cleans
up manually (the mobs never leave the tree on their own, so their `_exit_tree`
can't be relied on):

- Every spawned mob is tracked in a list and pruned via `tree_exited` when it
  dies in normal gameplay (guarded by `is_instance_valid()`).
- `clear_all_mobs()` frees the whole list and **resets respawn cooldowns**, so
  re-entering a region repopulates immediately instead of staying dead for the
  rest of each timer.
- It runs automatically on every **main-scene swap**: the manager hooks
  `get_tree().root.child_entered_tree` and clears when the entering node becomes
  `current_scene` (stray nodes parented to root, e.g. dropped coins, are
  ignored). Call `clear_all_mobs()` yourself any time you need a hard reset.

> **Note:** `smuggler_vessel` has no scene yet (`res://Mobs/Hostile/SmugglerVessel.scn`).
> The spawner skips missing scenes with a one-time warning; drop that scene in
> and it spawns automatically — no code change. `validate()` reports this on startup.

## How to retune the weights

Open `spawn_config.tres` in the Inspector and change numbers:

- Make a whole region busier/quieter → its entry in **Region Multipliers**.
- Shift what "high" vs "low" density means globally → **Density Weights**.
- Thin out / crowd the whole world at once → **Base Spawn Scalar**.
- Slower/faster repopulation → **Respawn Windows** (and which densities count as "fast").
- Fewer/more mobs alive at once → **Max Active Spawns**.

Per-mob, per-place tuning (a specific mob's density, time window, or group size in
a specific subzone) lives in `spawn_rules.json`.

## How to add content

### Add a mob
1. Make the mob scene, e.g. `res://Mobs/Hostile/NewMob.scn`.
2. In `MobTypes.gd`: add an `enum Mob` entry and a `REGISTRY` row
   (`"new_mob": { mob, klass, scene, display }`).
3. In `spawn_rules.json`: add `"new_mob": { density, time, group_min, group_max, location }`
   under whichever region/subzone rows it should appear in.

### Add a subzone
Add a new key under a region in `spawn_rules.json` (e.g. `"NCR": { "rooftops": { ... } }`)
and put mob rows under it. Drive the player into it with
`set_context("NCR", "rooftops")`. Optionally flag it under **Danger Subzones** in
the config.

### Add a region
1. Add a `"<Region>": { "<subzone>": { ...mobs... } }` block in `spawn_rules.json`.
2. Add `"<Region>": <multiplier>` to **Region Multipliers** in `spawn_config.tres`
   (unlisted regions default to 1.0, but be explicit).

`SpawnManager.validate()` runs on startup and warns about unknown mob keys,
missing scenes, unknown densities, and regions with no multiplier.

## Hooks (stubbed, ready to extend)

- **Danger zones** *(implemented as a weight bonus)* — `danger_subzones` (default
  `NCR/periphery_slum`, `CAR/deep_forest`) give `danger_mobs` (default `troll`,
  `commissioned_thug`) a `danger_weight_multiplier` (×1.5). Add zones/mobs in config.
- **Difficulty scaling** — `_apply_difficulty_scaling(group_size, ctx)` and
  `get_player_level()` in `SpawnManager.gd`. No-op until
  `difficulty_scaling_enabled`; wire to your progression to grow packs in
  high-level zones.
- **Elite swaps** — `_select_scene(mob_key)` calls `_elite_scene_for(mob_key)`
  when `elite_swap_enabled`. Return an elite variant scene path (e.g. a boss-tier
  thug at Batangas Port) to swap it in.
- **Event triggers** — `start_event(definition)` / `clear_event()`. `definition`
  is plain data:

  ```gdscript
  # Rally: extra journalists + order forces + snatchers
  SpawnManager.start_event({
      "weight_multipliers": { "journalist": 3.0, "order_forces": 2.0, "snatcher": 2.0 },
      "force_spawns": [ { "mob": "order_forces", "group_min": 3, "group_max": 5 } ],
  })

  # Festival: extra priests + roaming traders
  SpawnManager.start_event({
      "weight_multipliers": { "priest": 3.0, "roaming_trader": 3.0 },
  })
  ```

  `weight_multipliers` scale normal rolls until `clear_event()`; `force_spawns`
  fire once on the next evaluation. No event content ships by default — only the API.

## Out of scope

This system only spawns mobs. Loot tables, item drops, dialogue, quests, and
interaction logic are intentionally **not** handled here.
