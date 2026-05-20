extends Control

# Clean direct paths matching your CanvasLayer hierarchy
@onready var name_input: LineEdit = $VBoxContainer/CanvasLayer/PlayerNameInput
@onready var volume_slider: HSlider = $VBoxContainer/CanvasLayer/volume

func _ready() -> void:
	# 1. Manually instantiate your Rust class to bypass Autoload sync issues
	var save_manager = SaveManagerRust.new()
	var config_data = save_manager.load_config_json()
	
	if not config_data.is_empty():
		if config_data.has("player_name"):
			name_input.text = config_data["player_name"]
		
		# 2. Pass the saved value into the slider's audio helper function
		if config_data.has("volume"):
			volume_slider.set_volume_linear(config_data["volume"])
		print("UI fields successfully updated from fresh Rust instance.")
	else:
		print("Config file unestablished. Default settings applied.")

# Connected to your SaveSettingsButton
func _on_save_settings_button_pressed() -> void:
	var current_name = name_input.text
	var current_volume = volume_slider.value
	
	var save_manager = SaveManagerRust.new()
	save_manager.save_config_json(current_name, current_volume)
	print("Settings successfully saved via custom Rust instance!")

# Connected to your BackButton to change scenes back to Main Menu
func _on_back_button_pressed() -> void:
	# Adjust this path if your MainMenu file lives in a different folder!
	var main_menu_path = "res://Main/MainMenu.scn"
	
	if ResourceLoader.exists(main_menu_path):
		get_tree().change_scene_to_file(main_menu_path)
	else:
		print("Error: Scene file path not found at: ", main_menu_path)
