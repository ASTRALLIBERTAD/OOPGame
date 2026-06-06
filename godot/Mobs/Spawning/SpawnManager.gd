extends Node
## Data-driven mob spawner (autoload singleton: SpawnManager).
##
## NOTE: intentionally no `class_name` — the autoload named "SpawnManager" already
## registers that global identifier; a same-named class_name would conflict.
##
## Reads the player's current region + subzone + in-game time-of-day, looks up
## the matching rows in spawn_rules.json, and resolves them into spawns using
## the scalars in SpawnConfig. All spawn behaviour is data: this script contains
## no per-region/per-mob branching — it only interprets the table + config.
##
## Pipeline per evaluation tick (every config.evaluation_interval_seconds):
##   1. resolve_candidates(region, subzone, band)  -> rows whose time window
##      matches now, each with an effective spawn chance (pure / testable).
##   2. for each candidate not on respawn cooldown and under the population cap:
##         if randf() < chance: roll a group size in [group_min, group_max],
##         spawn it near the player, and start the row's respawn cooldown.
##
## Wiring: registered as an autoload (project.godot). Spawned mobs are parented
## to this manager, so they live under /root and survive scene changes — the
## manager owns their lifecycle and frees them via clear_all_mobs() on each
## main-scene swap. It self-discovers the player via the "player" group and stays
## idle while no player exists (e.g. in menus). Feed it context with
## set_context(region, subzone) from your world/streaming code.

signal mob_spawned(mob_key: String, instance: Node)
signal spawn_cycle_evaluated(region: String, subzone: String, band: String, groups_spawned: int)

## In-game time-of-day band.
enum Band { DAY, EVENING, NIGHT }

const SPAWNED_GROUP := "spawned_mob"
const PLAYER_GROUP := "player"

## Tunable config block. If left null a default SpawnConfig is created at runtime.
@export var config: SpawnConfig
## The spawn-rules table.
@export_file("*.json") var rules_path: String = "res://Mobs/Spawning/spawn_rules.json"
## Run the evaluation loop automatically. Turn off to drive evaluate() yourself.
@export var auto_run: bool = true
## Where spawned mobs are parented. Empty -> this manager (mobs persist under
## /root and are freed manually by clear_all_mobs()).
@export var spawn_parent_path: NodePath
## Cross-check the rules against the registry/config on startup and log issues.
@export var validate_on_ready: bool = true

## Current player location. Set these from your world/region-streaming code.
var current_region: String = "NCR"
var current_subzone: String = "urban_center"

var _rules: Dictionary = {}
var _rng := RandomNumberGenerator.new()
# "region|subzone|mob" -> Time.get_ticks_msec() value when it may spawn again.
var _cooldowns: Dictionary = {}
var _accum: float = 0.0
var _clock: float = 0.0            # seconds elapsed into the current day
var _time_override: int = -1       # a Band value, or -1 to use the clock
var _active_event: Dictionary = {} # event override (see start_event)
var _warned_missing: Dictionary = {}
var _spawned: Array = []           # mobs we spawned + own; freed manually on scene change


func _ready() -> void:
	if config == null:
		config = SpawnConfig.new()
	_rng.randomize()
	_load_rules()
	if validate_on_ready:
		validate()
	# Spawned mobs are parented to this autoload, so they never exit the tree on
	# their own — clear them on each main-scene swap (see _on_root_child_entered).
	get_tree().root.child_entered_tree.connect(_on_root_child_entered)
	set_process(auto_run)


func _process(delta: float) -> void:
	_clock = fmod(_clock + delta, maxf(config.day_length_seconds, 0.001))
	_accum += delta
	if _accum >= config.evaluation_interval_seconds:
		_accum = 0.0
		evaluate()


# ---------------------------------------------------------------------------
# Public context API
# ---------------------------------------------------------------------------

## Set the player's current region + subzone (call from region streaming).
func set_context(region: String, subzone: String) -> void:
	current_region = region
	current_subzone = subzone


## Force a specific time-of-day band, or pass -1 to resume the internal clock.
func set_time_of_day(band: int) -> void:
	_time_override = band


## The current time-of-day band (respects an override if one is set).
func current_band() -> int:
	if _time_override != -1:
		return _time_override
	var f := _clock / maxf(config.day_length_seconds, 0.001)
	if f < config.evening_start:
		return Band.DAY
	elif f < config.night_start:
		return Band.EVENING
	return Band.NIGHT


func band_name(band: int) -> String:
	match band:
		Band.DAY: return "day"
		Band.EVENING: return "evening"
		Band.NIGHT: return "night"
	return "day"


# ---------------------------------------------------------------------------
# Resolution (pure: reads rules + config, no side effects, easy to unit test)
# ---------------------------------------------------------------------------

## Return the spawn candidates for a region/subzone at a time band: every mob
## row whose time window matches, annotated with its effective spawn chance and
## group range. Does not roll or spawn anything.
func resolve_candidates(region: String, subzone: String, band: int) -> Array:
	var out: Array = []
	var region_rules: Dictionary = _rules.get(region, {})
	var subzone_rules: Dictionary = region_rules.get(subzone, {})
	var rmult := config.region_multiplier(region)
	var danger := config.is_danger(region, subzone)
	for mob_key in subzone_rules.keys():
		var row: Dictionary = subzone_rules[mob_key]
		var window := String(row.get("time", "24h"))
		if not _window_matches(window, band):
			continue
		var density := String(row.get("density", "low"))
		var chance := config.base_spawn_scalar * rmult * config.density_weight(density)
		if danger and config.danger_mobs.has(mob_key):
			chance *= config.danger_weight_multiplier      # danger-zone hook
		chance *= _event_weight_multiplier(mob_key)        # event hook
		out.append({
			"mob": mob_key,
			"density": density,
			"time": window,
			"chance": clampf(chance, 0.0, 1.0),
			"group_min": int(row.get("group_min", 1)),
			"group_max": int(row.get("group_max", 1)),
			"location": String(row.get("location", "")),
		})
	return out


## True if a mob's time window admits the given band.
## Allowed windows: day, evening, night, day_evening, evening_night, 24h.
func _window_matches(window: String, band: int) -> bool:
	match window:
		"24h": return true
		"day": return band == Band.DAY
		"evening": return band == Band.EVENING
		"night": return band == Band.NIGHT
		"day_evening": return band == Band.DAY or band == Band.EVENING
		"evening_night": return band == Band.EVENING or band == Band.NIGHT
	push_warning("SpawnManager: unknown time window '%s'" % window)
	return false


# ---------------------------------------------------------------------------
# Evaluation + spawning
# ---------------------------------------------------------------------------

## Run one spawn evaluation for the current context. Returns groups spawned.
## Safe to call manually (e.g. for tests) even when auto_run is off.
func evaluate() -> int:
	if _rules.is_empty():
		return 0
	# Stay idle when there is no player in the world (menus, loading, etc.).
	if get_tree().get_nodes_in_group(PLAYER_GROUP).is_empty():
		return 0

	var groups_spawned := 0
	groups_spawned += _process_event_force_spawns()   # event hook (one-shot)

	var band := current_band()
	for c in resolve_candidates(current_region, current_subzone, band):
		if _active_spawn_count() >= config.max_active_spawns:
			break
		var mob_key := String(c["mob"])
		if _on_cooldown(current_region, current_subzone, mob_key):
			continue
		if _rng.randf() < float(c["chance"]):
			var group_size := _roll_group(c)
			_spawn_group(mob_key, group_size, c)
			_set_cooldown(current_region, current_subzone, mob_key, String(c["density"]))
			groups_spawned += 1

	spawn_cycle_evaluated.emit(current_region, current_subzone, band_name(band), groups_spawned)
	return groups_spawned


func _roll_group(c: Dictionary) -> int:
	var lo := int(c["group_min"])
	var hi := maxi(lo, int(c["group_max"]))
	return _apply_difficulty_scaling(_rng.randi_range(lo, hi), c)


func _spawn_group(mob_key: String, count: int, ctx: Dictionary) -> void:
	var scene_path := _select_scene(mob_key)
	if scene_path == "" or not ResourceLoader.exists(scene_path):
		if not _warned_missing.has(mob_key):
			_warned_missing[mob_key] = true
			push_warning("SpawnManager: no scene for mob '%s' (%s) — skipping" % [mob_key, scene_path])
		return
	var parent := _resolve_spawn_parent()
	if parent == null:
		return
	var packed: PackedScene = load(scene_path)
	var origin := _spawn_origin()
	for _i in count:
		if _active_spawn_count() >= config.max_active_spawns:
			break
		var inst: Node = packed.instantiate()
		if inst is Node2D:
			(inst as Node2D).global_position = origin + _jitter()
		inst.add_to_group(SPAWNED_GROUP)
		# Tag provenance so other systems can read it. No interaction logic here.
		inst.set_meta("mob_key", mob_key)
		inst.set_meta("spawn_region", current_region)
		inst.set_meta("spawn_subzone", current_subzone)
		inst.set_meta("spawn_location", String(ctx.get("location", "")))
		parent.add_child(inst)
		_track(inst)
		mob_spawned.emit(mob_key, inst)


# ---------------------------------------------------------------------------
# Lifecycle / manual cleanup
#
# Mobs are parented to this autoload, so they never leave the tree on their own.
# We track them, free them on scene transitions, and prune the list as they die.
# ---------------------------------------------------------------------------

func _track(mob: Node) -> void:
	_spawned.append(mob)
	# A mob only ever exits the tree by being freed (gameplay death or
	# clear_all_mobs); prune it from our list when that happens.
	mob.tree_exited.connect(_on_mob_tree_exited.bind(mob), CONNECT_ONE_SHOT)


func _on_mob_tree_exited(mob: Node) -> void:
	_spawned.erase(mob)


## Free every mob this manager spawned and reset respawn cooldowns. Call on scene
## transitions — the mobs are parented to the autoload, so they would otherwise
## leak across scenes (they never exit the tree by themselves).
func clear_all_mobs() -> void:
	for mob in _spawned:
		if is_instance_valid(mob) and not mob.is_queued_for_deletion():
			mob.queue_free()
	_spawned.clear()
	# Reset cooldowns so re-entering a region repopulates right away instead of
	# staying empty for the remaining minutes of each respawn timer.
	_cooldowns.clear()


# Fires for every node added directly under /root. Only a real main-scene swap
# should trigger cleanup — not items parented to root (e.g. dropped coins), nor
# the autoloads themselves.
func _on_root_child_entered(node: Node) -> void:
	if node == self or node.scene_file_path.is_empty():
		return
	# current_scene is assigned during the swap; confirm it on the next idle frame.
	_check_scene_change.call_deferred(node)


func _check_scene_change(node: Node) -> void:
	if is_instance_valid(node) and get_tree().current_scene == node:
		clear_all_mobs()


# ---------------------------------------------------------------------------
# Difficulty / elite hooks (stubs — intentionally not implemented yet)
# ---------------------------------------------------------------------------

## Hook: scale group sizes up (or otherwise harden encounters) in high-level
## zones. No-op unless config.difficulty_scaling_enabled. Override get_player_level()
## and this method to implement scaling later.
func _apply_difficulty_scaling(group_size: int, _ctx: Dictionary) -> int:
	if not config.difficulty_scaling_enabled:
		return group_size
	# Example shape for later: grow the pack with player level in danger zones.
	#   if config.is_danger(current_region, current_subzone):
	#       group_size += int(get_player_level() / 10)
	return group_size


## Hook: pick the scene to spawn for a mob, allowing elite-variant swaps in
## high-level zones. Defaults to the base scene from the registry.
func _select_scene(mob_key: String) -> String:
	if config.elite_swap_enabled:
		var elite := _elite_scene_for(mob_key)
		if elite != "":
			return elite
	return MobTypes.scene_for(mob_key)


## Hook: return an elite-variant scene path for a mob, or "" for none.
func _elite_scene_for(_mob_key: String) -> String:
	return ""


## Stub for difficulty scaling. Wire this to your real player/progression code.
func get_player_level() -> int:
	return 1


# ---------------------------------------------------------------------------
# Event-trigger hook (stub API; ship no event content yet)
# ---------------------------------------------------------------------------

## Begin an event-based spawn override. The definition is plain data, e.g.:
##   { "weight_multipliers": { "journalist": 3.0, "order_forces": 2.0 },
##     "force_spawns": [ { "mob": "snatcher", "group_min": 3, "group_max": 5 } ] }
## weight_multipliers persist (scaling normal rolls) until clear_event();
## force_spawns fire once on the next evaluate(). See README for sample events
## (rally, festival). No event content is defined here — only the API.
func start_event(definition: Dictionary) -> void:
	_active_event = definition.duplicate(true)


## End the active event override.
func clear_event() -> void:
	_active_event = {}


func _event_weight_multiplier(mob_key: String) -> float:
	if _active_event.is_empty():
		return 1.0
	return float(_active_event.get("weight_multipliers", {}).get(mob_key, 1.0))


func _process_event_force_spawns() -> int:
	if _active_event.is_empty():
		return 0
	var forced: Array = _active_event.get("force_spawns", [])
	if forced.is_empty():
		return 0
	var n := 0
	for f in forced:
		if _active_spawn_count() >= config.max_active_spawns:
			break
		var mob_key := String(f.get("mob", ""))
		if mob_key == "":
			continue
		var lo := int(f.get("group_min", 1))
		var hi := maxi(lo, int(f.get("group_max", 1)))
		_spawn_group(mob_key, _rng.randi_range(lo, hi), f)
		n += 1
	# One-shot: consume forced spawns but keep weight_multipliers active.
	_active_event["force_spawns"] = []
	return n


# ---------------------------------------------------------------------------
# Respawn cooldowns
# ---------------------------------------------------------------------------

func _cooldown_key(region: String, subzone: String, mob: String) -> String:
	return "%s|%s|%s" % [region, subzone, mob]


func _on_cooldown(region: String, subzone: String, mob: String) -> bool:
	var k := _cooldown_key(region, subzone, mob)
	return _cooldowns.has(k) and Time.get_ticks_msec() < int(_cooldowns[k])


func _set_cooldown(region: String, subzone: String, mob: String, density: String) -> void:
	var lo: float
	var hi: float
	if config.is_fast_respawn(density):
		lo = config.fast_respawn_min_seconds
		hi = config.fast_respawn_max_seconds
	else:
		lo = config.respawn_min_seconds
		hi = config.respawn_max_seconds
	var secs := _rng.randf_range(lo, maxf(lo, hi))
	_cooldowns[_cooldown_key(region, subzone, mob)] = Time.get_ticks_msec() + int(secs * 1000.0)


# ---------------------------------------------------------------------------
# Placement helpers
# ---------------------------------------------------------------------------

func _resolve_spawn_parent() -> Node:
	if spawn_parent_path != NodePath() and has_node(spawn_parent_path):
		return get_node(spawn_parent_path)
	# Parent to the autoload itself: mobs live under /root and survive scene
	# changes, and this manager owns/frees them (see clear_all_mobs).
	return self


func _spawn_origin() -> Vector2:
	for p in get_tree().get_nodes_in_group(PLAYER_GROUP):
		if p is Node2D:
			var ang := _rng.randf() * TAU
			var dist := _rng.randf_range(config.spawn_radius_min, config.spawn_radius_max)
			return (p as Node2D).global_position + Vector2(cos(ang), sin(ang)) * dist
	return Vector2.ZERO


func _jitter() -> Vector2:
	return Vector2(
		_rng.randf_range(-config.member_jitter, config.member_jitter),
		_rng.randf_range(-config.member_jitter, config.member_jitter))


func _active_spawn_count() -> int:
	return get_tree().get_nodes_in_group(SPAWNED_GROUP).size()


# ---------------------------------------------------------------------------
# Loading + validation
# ---------------------------------------------------------------------------

func _load_rules() -> void:
	if not FileAccess.file_exists(rules_path):
		push_error("SpawnManager: rules file not found: " + rules_path)
		_rules = {}
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(rules_path))
	if typeof(parsed) != TYPE_DICTIONARY:
		push_error("SpawnManager: failed to parse rules JSON: " + rules_path)
		_rules = {}
		return
	# Accept either { "regions": {...} } or a bare { region: {...} } map.
	_rules = parsed.get("regions", parsed)


## Cross-check the loaded rules against the mob registry and config. Logs each
## distinct problem once (unknown mob key, missing scene, unknown density) and
## prints a summary. Expected to flag smuggler_vessel until its scene exists.
func validate() -> void:
	var rows := 0
	var problems := 0
	var bad_mobs := {}     # mob_key -> true (dedupe across regions)
	var bad_scenes := {}
	for region in _rules.keys():
		if not config.region_multipliers.has(region):
			push_warning("SpawnManager.validate: no region multiplier for '%s'" % region)
			problems += 1
		for subzone in _rules[region].keys():
			for mob_key in _rules[region][subzone].keys():
				rows += 1
				if not MobTypes.is_known(mob_key):
					if not bad_mobs.has(mob_key):
						bad_mobs[mob_key] = true
						push_warning("SpawnManager.validate: unknown mob key '%s'" % mob_key)
						problems += 1
				elif not ResourceLoader.exists(MobTypes.scene_for(mob_key)):
					if not bad_scenes.has(mob_key):
						bad_scenes[mob_key] = true
						push_warning("SpawnManager.validate: missing scene for '%s' -> %s" % [mob_key, MobTypes.scene_for(mob_key)])
						problems += 1
				var density := String(_rules[region][subzone][mob_key].get("density", ""))
				if not config.density_weights.has(density):
					push_warning("SpawnManager.validate: unknown density '%s' (%s/%s/%s)" % [density, region, subzone, mob_key])
					problems += 1
	print("SpawnManager.validate: %d rows checked across %d region(s), %d distinct problem(s)." % [rows, _rules.size(), problems])
