extends CenterContainer

@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
@onready var _slots: Array = get_children()

var _index: int
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_inv.update.connect(_updated)

	pass # Replace with function body.

func _updated():
	var inventory_slot : InvSlot = _inv.slots[_index]
	_slots[0].update_to_slot(inventory_slot)
	call_deferred("_refresh_player_item", inventory_slot)

func _refresh_player_item(inventory_slot: InvSlot):
	var player = get_tree().get_first_node_in_group("player")
	if player:
		player.item_right.set_item(inventory_slot.item)

func selected_item(item_index):
	_index = item_index
	var inventory_slot : InvSlot = _inv.slots[_index]
	_slots[0].update_to_slot(inventory_slot)
	var player = get_tree().get_first_node_in_group("player")
	if player:
		player.item_right.set_item(inventory_slot.item)
		_inv.emit_signal("update")

func get_inv() -> Inventory:
	return _inv
