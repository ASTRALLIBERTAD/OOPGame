extends Control
@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
@onready var _slots: Array = $NinePatchRect/GridContainer.get_children()

var _tapped_slot :Array= []
var _first_slot : int = -1

@onready var _tex: Sprite2D = $NinePatchRect/Sprite2D
var _held_item: Texture = null

func _ready() -> void:
	_inv.update.connect(_update_slots)
	_update_slots()
	for b in range(_slots.size()):
		var btn = _slots[b].get_node("CenterContainer/Panel/Button") as Button
		btn.connect("pressed", func() -> void: _on_slot_pressed(b))

func _on_slot_pressed(index: int) -> void:
	print("Slot pressed:", index)

	if not _tapped_slot.has(index):
		_tapped_slot.append(index)
		var item = _inv.slots[index].item
		if item:
			_held_item = item.icon
			_tex.texture = _held_item
			_tex.visible = true
	else:
		# Unselecting the slot
		_first_slot = -1
		_tapped_slot.clear()
		_held_item = null
		_tex.visible = false
		return

	if _first_slot == -1:
		if _inv.slots[index].item.name:
			_first_slot = index
			return

	# Cancel the operation if the first slot has no item or name is empty
	var first_item = _inv.slots[_first_slot].item
	if first_item == null or first_item.name == "" or first_item.name == null:
		print("First slot has no valid item name. Cancelling.")
		_first_slot = -1
		_tapped_slot.clear()
		_held_item = null
		_tex.visible = false
		return

	if _tapped_slot.size() == 2:
		_swap_items(_first_slot, index)

		_tapped_slot.clear()
		_first_slot = -1
		_held_item = null
		_tex.visible = false


func _physics_process(_delta: float) -> void:
	if _held_item:
		_tex.global_position = get_viewport().get_mouse_position()
		_tex.visible = true

func _swap_items(index1:int, index2:int):
	var t = _inv.slots[0].item as Collectibles
	_inv.insert(t, index1, index2)

func _update_slots():
	for i in range(min(_inv.slots.size(), _slots.size())):
		_slots[i].update(_inv.slots[i])
		print("happpe")
