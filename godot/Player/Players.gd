extends Rustplayer

func _on_timer_timeout() -> void:
	$Camera2D.position_smoothing_enabled = true
	$Camera2D.set_position_smoothing_enabled(2)
	pass # Replace with function body.

func _on_inventory_pressed() -> void:
	open_close()

func full_or_not(item: Collectibles = null) -> bool:
	for i in range(12):
		if inv.slots[i].item.name == "":
			return false
		if item and item.stackable and inv.slots[i].item.name == item.name:
			return false
	return true

func player():
	pass

#func _enter_tree() -> void:
	#SaveManager.player_node = self

func _on_piso_changed(new_total: int) -> void:
	%piso.text = "Piso: " + str(new_total)


func _on_message(message: String) -> void:
	%ChatHUD.add_message(message)
