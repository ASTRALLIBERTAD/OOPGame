extends Node2D
@onready var _tile_set = $Terrain1


func _ready() -> void:
	$AutoSave.start()



func _process(delta: float) -> void:
	var tile_cord = _player_cord()
	%coords.text = "Coordinates: %d,%d" % [tile_cord.x, tile_cord.y * -1]



func _on_auto_save_timeout() -> void:
	SaveManager.auto_save()
	pass # Replace with function body.

func _player_cord():
	var cord = _tile_set.local_to_map($PLAYERS.global_position)
	var local_position = _tile_set.to_local(cord)
	return local_position
	#done in Rust
	pass

func _on_button_pressed() -> void:
	var date = str(Time.get_datetime_string_from_system())
	date = date.replace(":", "-")
	SaveManager.save_game()
	$Control/TouchControls.visible = false
	get_tree().paused = true
	# In your Godot script

func _on_loading_pressed() -> void:
	print(OS.get_user_data_dir())
	var yt = SaveManager.get_os()
	$po.text = yt

	$osl.text = OS.get_name()
	get_tree().paused = false
	pass


func _on_saving_time_timeout() -> void:
	get_tree().paused = false
	SaveManager.save_game()
	get_tree().change_scene_to_file("res://SaveAndLoad/LoadMenu.tscn")
	queue_free()
	queue_redraw()
	pass # Replace with function body.


func _on_menu_pressed() -> void:
	%TouchControls.visible = false
	get_tree().paused = true
	%Panel.visible = true
	pass # Replace with function body.


func _on_save_pressed() -> void:
	%TouchControls.visible = false
	%Panel.visible = false
	%SavingTime.start()
	pass # Replace with function body.


func _on_back_pressed() -> void:
	%TouchControls.visible = true
	%Panel.visible = false
	get_tree().paused = false
	pass # Replace with function body.
