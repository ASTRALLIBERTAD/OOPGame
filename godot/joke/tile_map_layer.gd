extends TileMapLayer

# Lookup table: neighbor mask → atlas coord (x,y) in tileset
var _grass_bitmask_map = {
	255: Vector2i(0, 0), # surrounded
	2:   Vector2i(1, 0), # only top
	8:   Vector2i(2, 0), # only left
	16:  Vector2i(3, 0), # only right
	32:  Vector2i(4, 0), # only bottom

	(2+8):   Vector2i(0, 1), # top-left corner
	(2+16):  Vector2i(1, 1), # top-right corner
	(32+8):  Vector2i(2, 1), # bottom-left corner
	(32+16): Vector2i(3, 1), # bottom-right corner
	# …continue filling depending on your sheet layout
}

# Directions and their mask values
var _directions = {
	Vector2i(-1, -1): 1,
	Vector2i(0, -1): 2,
	Vector2i(1, -1): 4,
	Vector2i(-1, 0): 8,
	Vector2i(1, 0): 16,
	Vector2i(-1, 1): 32,
	Vector2i(0, 1): 64,
	Vector2i(1, 1): 128,
}

# The source_id of your grass tileset (check TileSet inspector!)
const SOURCE_ID = 2

func _ready() -> void:
	# Example: generate a 10x10 grass patch
	for x in range(10):
		for y in range(10):
			_autotile(Vector2i(x, y))

func _autotile(pos: Vector2i) -> void:
	var mask := 0
	for dir in _directions.keys():
		var neighbor = pos + dir
		if _is_grass(neighbor):
			mask |= _directions[dir]
	# Pick atlas coords from lookup, fallback if missing
	var tile_coord: Vector2i = _grass_bitmask_map.get(mask, Vector2i(0, 0))
	# Place tile
	set_cell(pos, SOURCE_ID, tile_coord)
# Dummy function: replace with your world data check

func _is_grass(pos: Vector2i) -> bool:
	return true
