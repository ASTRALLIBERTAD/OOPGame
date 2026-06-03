extends Rustplayer

func _ready() -> void:
	# This GDScript _ready shadows Rustplayer::ready(), and super() cannot reach
	# the Rust base. init_player() is a #[func] exposing the full Rust setup
	# (add_to_group("player"), camera, HUD, register_rpcs), so calling it here
	# gives identical initialization.
	init_player()
	# TEST: drop an Iron Helmet into the inventory so we can equip it via the UI
	inv.insert(preload("res://Collectibles/items/helmet_item.tres"), -1, -1)

func _on_timer_timeout() -> void:
	$Camera2D.position_smoothing_enabled = true
	$Camera2D.set_position_smoothing_enabled(2)
	pass # Replace with function body.

func _on_inventory_pressed() -> void:
	open_close()

func full_or_not() -> bool:
	var is_full = true
	for i in inv.slots:
		if i.item.name == "":
			is_full = false
			return is_full
	return is_full

func player():
	pass

#func _enter_tree() -> void:
	#SaveManager.player_node = self

func _on_piso_changed(new_total: int) -> void:
	%piso.text = "Piso: " + str(new_total)


func _on_message(message: String) -> void:
	%ChatHUD.add_message(message)
