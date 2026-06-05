extends Control

@onready var name_input: LineEdit = $VBoxContainer/CanvasLayer/PlayerNameInput
@onready var volume_slider: HSlider = $VBoxContainer/CanvasLayer/volume

func _ready() -> void:
	var saved_name = RustSaveManager1.get_config_player_name()
	var saved_volume = RustSaveManager1.get_config_volume()
	
	if saved_name != "":
		name_input.text = saved_name
	else:
		print("Player name field fallback applied or config unestablished.")
		
	if volume_slider.has_method("set_volume_linear"):
		volume_slider.set_volume_linear(saved_volume)
	else:
		volume_slider.value = saved_volume
		
	print("UI fields successfully updated from individual Rust getters.")

func _on_save_settings_button_pressed() -> void:
	var current_name = name_input.text
	var current_volume = volume_slider.value
	
	var save_manager = RustSaveManager1
	if name_input.text == "":
		name_input.placeholder_text = "Enter a name"
		return
	save_manager.save_config_json(current_name, current_volume)
	print("Settings successfully saved via custom Rust instance!")

func _on_back_button_pressed() -> void:
	get_tree().change_scene_to_file("res://Main/MainMenu.scn")
