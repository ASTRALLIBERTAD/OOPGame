extends TileMapLayer

var _moisture = FastNoiseLite.new()
var _temperature = FastNoiseLite.new()
var _altitude = FastNoiseLite.new()

@onready var _player = %PLAYERS

var _height: int = 32
var _width: int = 32

var _loaded_chunks := []

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_moisture.seed = randi()
	_temperature.seed = randi()
	_altitude.seed = randi()

	# Adjust this value to change the 'smoothness' of the map; lower values mean more smooth noise
	_altitude.frequency = 0.01

	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	var player_tile_pos := local_to_map(_player.position)
	# Generate the chunk at the player's position
	_generate_chunk(player_tile_pos)
	# Unload chunks that are too far away.
	# Note: Not needed for smaller projects but if you are loading a bigger tilemap it's good practice
	_unload_distant_chunks(player_tile_pos)
	pass


func _generate_chunk(pos):
	for x in range(_width):
		for y in range(_height):
			# Generate noise values for moisture, temperature, and altitude
			var moist := _moisture.get_noise_2d(pos.x - (_width/2) + x, pos.y - (_height/2) + y) * 10 # Values between -10 and 10
			var temp := _temperature.get_noise_2d(pos.x - (_width/2) + x, pos.y - (_height/2) + y) * 10
			var alt := _altitude.get_noise_2d(pos.x - (_width/2) + x, pos.y - (_height/2) + y) * 10
			# Set the cell based on altitude; adjust for different tile types
			# Need to evenly distribute -10 -> 10 to 0 -> 4....  This can be done by first adding 10
			# Gets values from 0 -> 20... Then we will multiply by 3/20 in order to remap it to 0 -> 3
			# vvv
			if alt < 0: # Arbitrary sea level value (choosing 0 will mean roughly 1/2 the world is ocean)
				set_cell( Vector2i(pos.x - (_width/2) + x, pos.y - (_height/2) + y), 0, Vector2(4, 10))
			else: # You can add other logic like making beaches by setting x-coord to whatever beach atlas x-coord is when the alt is between 0 and 0.5 or something
				set_cell( Vector2i(pos.x - (_width/2) + x, pos.y - (_height/2) + y), 0, Vector2(0,1))


			if Vector2i(pos.x, pos.y) not in _loaded_chunks:
				_loaded_chunks.append(Vector2i(pos.x, pos.y))

func _unload_distant_chunks(player_pos):
	# Set the distance threshold to at least 2 times the width to limit visual glitches
	# Higher values unload chunks further away
	var unload_distance_threshold := (_width * 2) + 1

	for chunk in _loaded_chunks:
		var distance_to_player = _get_dist(chunk, player_pos)
		if distance_to_player > unload_distance_threshold:
			_clear_chunk(chunk)
			_loaded_chunks.erase(chunk)

func _clear_chunk(pos):
	for x in range(_width):
		for y in range(_height):
			set_cell( Vector2i(pos.x - (_width/2) + x, pos.y - (_height/2) + y), -1, Vector2(-1, -1), -1)


func _get_dist(p1, p2):
	var resultant = p1 - p2
	return sqrt(resultant.x ** 2 + resultant.y ** 2)
