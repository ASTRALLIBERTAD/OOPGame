class_name MobTypes
extends RefCounted
## Canonical registry of every spawnable mob.
##
## This is the single source of truth that maps a mob's data key (used in
## spawn_rules.json) to its enum value, gameplay class, scene file, and a
## human-readable display name. Add a new mob here AND in spawn_rules.json.
##
## Mob types are defined as an enum (requirement) plus a registry that carries
## the structural data the spawner needs. Keys are snake_case and must match the
## mob keys used inside spawn_rules.json.

## Every mob type in the game.
enum Mob {
	FARMERS,
	OFWS,
	PRIEST,
	JOURNALIST,
	ROAMING_TRADER,
	SNATCHER,
	COMMISSIONED_THUG,
	CORRUPTION_BROKER,
	ORDER_FORCES,
	SMUGGLER_VESSEL,
	TROLL,
}

## Gameplay disposition. Spawn weighting / danger-zone rules key off this.
enum Klass { PASSIVE, NEUTRAL, HOSTILE }

## key -> { mob, klass, scene, display }
## NOTE: SmugglerVessel.scn does not exist in the project yet. The spawner skips
## any mob whose scene is missing (with a one-time warning); drop the scene at
## the path below and it starts spawning with no code change.
const REGISTRY := {
	"farmers": {
		"mob": Mob.FARMERS, "klass": Klass.PASSIVE,
		"scene": "res://Mobs/Passive/Farmer.scn", "display": "Magsasaka (Farmer)",
	},
	"ofws": {
		"mob": Mob.OFWS, "klass": Klass.PASSIVE,
		"scene": "res://Mobs/Passive/Ofw.scn", "display": "Balikbayan (OFW)",
	},
	"priest": {
		"mob": Mob.PRIEST, "klass": Klass.PASSIVE,
		"scene": "res://Mobs/Passive/Priest.scn", "display": "Pari / Pastor (Priest)",
	},
	"journalist": {
		"mob": Mob.JOURNALIST, "klass": Klass.NEUTRAL,
		"scene": "res://Mobs/Neutral/Journalist.scn", "display": "Mamamahayag (Journalist)",
	},
	"roaming_trader": {
		"mob": Mob.ROAMING_TRADER, "klass": Klass.NEUTRAL,
		"scene": "res://Mobs/Neutral/RoamingTrader.scn", "display": "Tagapamagitan (Roaming Trader)",
	},
	"snatcher": {
		"mob": Mob.SNATCHER, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/Snatcher.scn", "display": "Snatcher",
	},
	"commissioned_thug": {
		"mob": Mob.COMMISSIONED_THUG, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/CommissionedThug.scn", "display": "Komisyon Goon (Commissioned Thug)",
	},
	"corruption_broker": {
		"mob": Mob.CORRUPTION_BROKER, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/CorruptionBroker.scn", "display": "Trapo Fixer (Corruption Broker)",
	},
	"order_forces": {
		"mob": Mob.ORDER_FORCES, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/OrderForce.scn", "display": "Puersa ng Orden (Order Forces)",
	},
	"smuggler_vessel": {
		"mob": Mob.SMUGGLER_VESSEL, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/SmugglerVessel.scn", "display": "Smuggler Vessel",
	},
	"troll": {
		"mob": Mob.TROLL, "klass": Klass.HOSTILE,
		"scene": "res://Mobs/Hostile/Troll.scn", "display": "Troll Army",
	},
}


## All registered mob keys.
static func keys() -> Array:
	return REGISTRY.keys()


## True if the key is a known mob.
static func is_known(key: String) -> bool:
	return REGISTRY.has(key)


## Scene path for a mob key, or "" if unknown.
static func scene_for(key: String) -> String:
	return String(REGISTRY.get(key, {}).get("scene", ""))


## Gameplay class for a mob key (defaults to HOSTILE if unknown).
static func klass_for(key: String) -> int:
	return int(REGISTRY.get(key, {}).get("klass", Klass.HOSTILE))


## Display name for a mob key.
static func display_for(key: String) -> String:
	return String(REGISTRY.get(key, {}).get("display", key))


## Mob enum value for a key (or -1 if unknown).
static func key_to_mob(key: String) -> int:
	return int(REGISTRY.get(key, {}).get("mob", -1))


## Reverse lookup: enum value -> data key (or "" if not found).
static func mob_to_key(mob: int) -> String:
	for key in REGISTRY.keys():
		if int(REGISTRY[key]["mob"]) == mob:
			return key
	return ""
