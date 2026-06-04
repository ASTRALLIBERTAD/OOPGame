extends HSlider

@export var bus_name: String = "Master"
var bus_index: int

func _ready() -> void:
	bus_index = AudioServer.get_bus_index(bus_name)
	value_changed.connect(_on_value_changed)
	
	
	value = db_to_linear(AudioServer.get_bus_volume_db(bus_index))

func _on_value_changed(new_value: float) -> void:
	AudioServer.set_bus_volume_db(bus_index, linear_to_db(new_value))


func set_volume_linear(new_value: float) -> void:
	value = new_value
	AudioServer.set_bus_volume_db(bus_index, linear_to_db(new_value))
