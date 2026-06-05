extends Control

@onready var _inv: Inventory = preload("res://Collectibles/items/inventory.res")
@onready var _slots: Array = $NinePatchRect/GridContainer.get_children()
@onready var _armor_slots: Array = $VBoxContainer.get_children()

var _tapped_slot : Array = []
var _first_slot : int = -1

@onready var _tex: Sprite2D = $NinePatchRect/Sprite2D
var _held_item: Texture = null

func _ready() -> void:
	_inv.update.connect(_update_slots)
	_update_slots()
	
	for b in range(_slots.size()):
		var btn = _slots[b].get_node("CenterContainer/Panel/Button") as Button
		btn.connect("pressed", func() -> void: _on_slot_pressed(b))
	for a in range(_armor_slots.size()):
		var btn = _armor_slots[a].get_node("CenterContainer/Panel/Button") as Button
		var global_armor_index = _slots.size() + a 
		btn.connect("pressed", func() -> void: _on_slot_pressed(global_armor_index))

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
		_reset_selection()
		return
	if _first_slot == -1:
		if _inv.slots[index].item.name:
			_first_slot = index
			return
	var first_item = _inv.slots[_first_slot].item
	if first_item == null or first_item.name == "" or first_item.name == null:
		print("First slot has no valid item name. Cancelling.")
		_reset_selection()
		return

	if _tapped_slot.size() == 2:
		_swap_items(_first_slot, index)
		_reset_selection()

func _physics_process(_delta: float) -> void:
	if _held_item:
		_tex.global_position = get_viewport().get_mouse_position()
		_tex.visible = true

func _swap_items(index1: int, index2: int):
	var dummy_item = _inv.slots[0].item as Collectibles
	_inv.insert(dummy_item, index1, index2)

func _update_slots():
	for i in range(min(_inv.slots.size(), _slots.size())):
		_slots[i].update(_inv.slots[i])
		
	for a in range(_armor_slots.size()):
		var global_armor_index = _slots.size() + a
		if global_armor_index < _inv.slots.size():
			_armor_slots[a].update(_inv.slots[global_armor_index])

func _reset_selection() -> void:
	_first_slot = -1
	_tapped_slot.clear()
	_held_item = null
	_tex.visible = false

func _on_drop_button_pressed() -> void:
	if _first_slot != -1:
		var slot_data = _inv.slots[_first_slot]
		
		if slot_data and slot_data.item and slot_data.item.name:
			if not slot_data.item.has_meta("source_path"):
				print("No source path on item, cannot drop.")
				return
			
			var path = (slot_data.item.get_meta("source_path") as String).replace(".res", ".scn")
			print("the path is" + path)
			
			if FileAccess.file_exists(path):
				var item_scene = load(path)
				if item_scene:
					var drop_count = slot_data.item.amount
					
					var player = get_node("../../../")
					var item_instance = item_scene.instantiate()
					item_instance.global_position = player.global_position + Vector2(50, 0)
					
					var dropped_item = item_instance.item.duplicate()
					dropped_item.amount = drop_count
					dropped_item.set_meta("source_path", slot_data.item.get_meta("source_path"))
					item_instance.item = dropped_item
					
					get_tree().root.get_node("main").add_child(item_instance)
					
					var empty_item = Collectibles.new()
					slot_data.item = empty_item
					
					_inv.update.emit()
					_reset_selection()
			else:
				print("Error: Scene file doesn't exist at path: ", path)
