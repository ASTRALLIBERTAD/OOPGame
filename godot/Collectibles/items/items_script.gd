extends StaticBody2D

@export var item: Collectibles
var player = null

func _on_area_2d_body_entered(body: Node2D) -> void:
	if body.has_method("player"):
		if body.has_method("full_or_not"):
			if body.full_or_not() == false or item.stackable == true:
				print(body.full_or_not())
				player = body
				player_collect()
				await get_tree().create_timer(0.1).timeout
				self.queue_free()
				if multiplayer.is_server():
					rpc("self_destroy")
			else:
				print("inventory is full")

@rpc("call_remote", "reliable")
func self_destroy():
	self.queue_free()

func player_collect():	player.collect_items(item, -1)
