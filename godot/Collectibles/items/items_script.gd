extends StaticBody2D

@export var item: Collectibles
var _player = null

func _on_area_2d_body_entered(body: Node2D) -> void:
	if body.has_method("player"):
		_player = body
		if item.name == "piso":
			_collect_piso()
			await get_tree().create_timer(0.1).timeout
			queue_free()
			if multiplayer.is_server():
				rpc("self_destroy")
		else:
			if body.has_method("full_or_not"):
				if body.full_or_not() == false or item.stackable == true:
					_player_collect()
					await get_tree().create_timer(0.1).timeout
					queue_free()
					if multiplayer.is_server():
						rpc("self_destroy")
				else:
					EventBus.message.emit("Inventory is full")

@rpc("call_remote", "reliable")
func self_destroy():
	queue_free()

func _player_collect():
	_player.collect_items(item, -1)

func _collect_piso():
	_player.add_piso(item.amount)
	EventBus.message.emit("+" + str(item.amount) + " piso")

func get_player():
	return _player
