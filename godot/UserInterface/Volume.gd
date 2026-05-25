extends HSlider

<<<<<<< Updated upstream
@export var bus_name: String
var bus_index: int
=======
@export
var bus_name: String

var _bus_index: int
>>>>>>> Stashed changes

func _ready() -> void:
	_bus_index = AudioServer.get_bus_index(bus_name)
	value_changed.connect(_on_value_changed)
	
	# Default fallback fetch from the current engine state
	value = db_to_linear(AudioServer.get_bus_volume_db(bus_index))

<<<<<<< Updated upstream
func _on_value_changed(new_value: float) -> void:
	AudioServer.set_bus_volume_db(bus_index, linear_to_db(new_value))

# The master script calls this helper when loading your file data to set both
# the slider's physical position and update the actual decibel audio bus.
func set_volume_linear(new_value: float) -> void:
	value = new_value
	AudioServer.set_bus_volume_db(bus_index, linear_to_db(new_value))
=======
	value = db_to_linear(
		AudioServer.get_bus_volume_db(_bus_index)
	)

func _on_value_changed(value: float) -> void:
	AudioServer.set_bus_volume_db(
		_bus_index,
		linear_to_db(value)
	)
>>>>>>> Stashed changes
