extends Sprite2D


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


func _on_area_2d_area_entered(area) -> void:
	if area.has_method("weapon_damage"):
		area.weapon_damage(1)
		print("damadad")
		pass
	else :
		print("nonoo")
	pass # Replace with function body.
