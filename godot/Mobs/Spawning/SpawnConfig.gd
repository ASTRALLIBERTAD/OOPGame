class_name SpawnConfig
extends Resource
## The single tunable config block for the whole spawn system.
##
## Every balance number lives here so designers can retune the game from one
## Resource (spawn_config.tres) in the Inspector — no code edits. The big
## per-region/subzone/mob table lives separately in spawn_rules.json; this
## Resource only holds the global scalars that table is interpreted against.
##
## Effective per-evaluation spawn chance for a mob row is:
##     base_spawn_scalar
##   * region_multiplier(region)          # baseline encounter rate of the region
##   * density_weight(row.density)        # subzone density weight of that mob
##   * (danger_weight_multiplier if applicable)
##   * (event multiplier if an event is active)
## ...clamped to [0, 1]. After a successful spawn the (subzone, mob) goes on a
## respawn cooldown rolled from the respawn window below.

## Region -> baseline encounter-rate multiplier (High tiers ~1.0, Low ~0.35).
@export var region_multipliers: Dictionary = {
	"NCR": 1.0,
	"CALABARZON": 1.0,
	"Ilocos": 0.6,
	"Bicol": 0.6,
	"CAR": 0.35,
}

## Qualitative density -> numeric spawn weight.
@export var density_weights: Dictionary = {
	"very_low": 0.1,
	"low": 0.3,
	"med": 0.6,
	"high": 1.0,
}

## Global dial over all spawn chances. 1.0 = use region x density as-is.
## Lower this to thin out the world, raise it to make encounters more frequent.
@export var base_spawn_scalar: float = 1.0

## How often (seconds) the spawner re-evaluates the current region/subzone.
@export var evaluation_interval_seconds: float = 5.0

@export_group("Respawn Windows")
## Default respawn cooldown range (seconds) per spawn slot. Spec: 5–10 min.
@export var respawn_min_seconds: float = 300.0
@export var respawn_max_seconds: float = 600.0
## Faster respawn range used in high-density slots. Spec: 2–4 min.
@export var fast_respawn_min_seconds: float = 120.0
@export var fast_respawn_max_seconds: float = 240.0
## Densities whose weight is >= this density's weight use the fast window.
@export_enum("very_low", "low", "med", "high") var fast_respawn_density: String = "high"

@export_group("Population")
## Hard cap on concurrently alive spawned mobs (across all subzones).
@export var max_active_spawns: int = 40
## A spawn group's centre is placed this far from the player (ring), in px.
@export var spawn_radius_min: float = 250.0
@export var spawn_radius_max: float = 600.0
## Per-member scatter around the group centre, in px.
@export var member_jitter: float = 48.0

@export_group("Danger Zones")
## "Region/subzone" entries flagged high-danger (urban slums, deep forest).
## Tougher hostiles get a weight bonus here.
@export var danger_subzones: PackedStringArray = [
	"NCR/periphery_slum",
	"CAR/deep_forest",
]
## Mob keys treated as the "tougher hostiles" for the danger-zone bonus.
@export var danger_mobs: PackedStringArray = ["troll", "commissioned_thug"]
## Weight multiplier applied to danger_mobs inside danger_subzones.
@export var danger_weight_multiplier: float = 1.5

@export_group("Time Of Day")
## Length of one full in-game day/night cycle, in real seconds.
@export var day_length_seconds: float = 1200.0
## Fraction of the day [0,1) at which "evening" begins.
@export var evening_start: float = 0.5
## Fraction of the day [0,1) at which "night" begins.
@export var night_start: float = 0.75

@export_group("Difficulty / Elite Hooks (stubs)")
## When true, SpawnManager._apply_difficulty_scaling() may grow group sizes.
@export var difficulty_scaling_enabled: bool = false
## When true, SpawnManager._select_scene() may swap in an elite variant scene.
@export var elite_swap_enabled: bool = false


## Region encounter-rate multiplier (defaults to 1.0 for unlisted regions).
func region_multiplier(region: String) -> float:
	return float(region_multipliers.get(region, 1.0))


## Numeric weight for a qualitative density name (0.0 if unknown).
func density_weight(density: String) -> float:
	return float(density_weights.get(density, 0.0))


## True if "region/subzone" is flagged as a danger zone.
func is_danger(region: String, subzone: String) -> bool:
	return danger_subzones.has("%s/%s" % [region, subzone])


## True if a density should use the fast respawn window.
func is_fast_respawn(density: String) -> bool:
	return density_weight(density) >= density_weight(fast_respawn_density)
