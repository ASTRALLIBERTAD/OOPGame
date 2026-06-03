extends HBoxContainer
@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
@onready var _slots: Array = get_children()

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_inv.update.connect(_updated)
	_updated()
	for b in range(_slots.size()):
		var btn = _slots[b].get_node("CenterContainer/Panel/Button") as Button
		btn.connect("pressed", func() -> void: _on_slot_pressed(b))
	pass # Replace with function body.

func _on_slot_pressed(index: int) -> void:
	$"../CenterContainer".selected_item(index)
	print("Slot pressed:", index)

func _updated():
	for i in range(_slots.size()):
		var inventory_slot : InvSlot = _inv.slots[i]
		_slots[i].update_to_slot(inventory_slot)
		print("happpe")
