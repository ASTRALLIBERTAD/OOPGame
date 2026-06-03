extends Control
const ArmorUiScript := preload("res://UserInterface/armor_ui.gd")
@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
@onready var _slots: Array = $NinePatchRect/GridContainer.get_children()

var _tapped_slot :Array= []
var _first_slot : int = -1

@onready var _tex: Sprite2D = $NinePatchRect/Sprite2D
var _held_item: Texture = null

var _armor_ui

func _ready() -> void:
	_inv.update.connect(_update_slots)
	_update_slots()
	for b in range(_slots.size()):
		var btn = _slots[b].get_node("CenterContainer/Panel/Button") as Button
		btn.connect("pressed", func() -> void: _on_slot_pressed(b))

	# Build the armor UI at runtime (no scene/editor steps) and route its slot
	# taps through our existing tap-to-select state.
	_armor_ui = ArmorUiScript.new()
	add_child(_armor_ui)
	_armor_ui.armor_slot_pressed.connect(_on_armor_slot_pressed)

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

func _reset_selection() -> void:
	_first_slot = -1
	_tapped_slot.clear()
	_held_item = null
	_tex.visible = false

func _on_armor_slot_pressed(slot_index: int) -> void:
	# An inventory item is selected -> try to equip it onto this armor slot.
	if _first_slot != -1:
		var item := _inv.slots[_first_slot].item as Collectibles
		if item != null and item.is_armor() \
				and item.get_armor_piece().get_slot_index() == slot_index:
			# `=` not `:=`: _armor_ui is untyped, so the parser can't infer the
			# try_equip() return type.
			var displaced = _armor_ui.try_equip(item)
			_inv.slots[_first_slot].clear_item()
			# Whatever was previously equipped here goes back to the inventory.
			if displaced != null and String(displaced.get_name()) != "":
				_inv.insert(displaced, -1, -1)   # emits inventory update
			_update_slots()
		_reset_selection()
	else:
		# Nothing selected -> unequip this slot back into the inventory.
		# `=` not `:=`: untyped _armor_ui, so unequip_slot()'s type can't be inferred.
		var removed = _armor_ui.unequip_slot(slot_index)
		if removed != null and String(removed.get_name()) != "":
			_inv.insert(removed, -1, -1)
			_update_slots()
