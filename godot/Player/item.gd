extends Sprite2D
#
#@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
#@onready var _slots: Array = get_children()
#
#var _index: int
## Called when the node enters the scene tree for the first time.
#func _ready() -> void:
	#_inv.update.connect(_updated)
#
	#pass # Replace with function body.
#
#func _updated():
	#var inventory_slot : InvSlot = _inv.slots[_index]
	#_slots[0].update_to_slot(inventory_slot)
	#print("happpe")
#
#func selected_item(item_index):
	#_index = item_index
	#var inventory_slot : InvSlot = _inv.slots[_index]
	#_slots[0].update_to_slot(inventory_slot)
	#print("happpe")
#
#func get_inv() -> Inventory:
	#return _inv
