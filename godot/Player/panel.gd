extends Panel

@onready var _item_visual: Sprite2D = $item

func update_to_slot(slot: InvSlot):
	if !slot.item.name:
		_item_visual.visible= false
	else:
		_item_visual.visible = true
		_item_visual.texture = slot.item.icon

func _on_button_pressed() -> void:
	#out.emit()
	pass # Replace with function body.
