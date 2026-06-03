extends Control
## Runtime-built armor equipment UI.
##
## Instanced by item_slot.gd via add_child() — no scene file, no editor steps.
## Shows the 4 armor slots (Helmet / Body / Leggings / Boots), each with the
## equipped item's icon plus its defense and speed_modifier stats. Shares the
## same ArmorInventory resource instance the player uses (preloaded .tres),
## mirroring how item_slot.gd shares inventory.res.

const SLOT_NAMES: Array[String] = ["Helmet", "Body", "Leggings", "Boots"]

# Same instance the player's `armor_inv` export points to.
var _armor: ArmorInventory = preload("res://Player/armor_inventory.tres")

# The Collectibles equipped through the UI, per slot (for icon display and for
# returning to the inventory on unequip). null = slot empty / unknown source.
var _equipped_items: Array = [null, null, null, null]

var _icons: Array = []
var _def_labels: Array = []
var _spd_labels: Array = []
var _buttons: Array = []

## Emitted when an armor slot is tapped. item_slot.gd owns the selection state,
## so it decides whether this means "equip selected item" or "unequip".
signal armor_slot_pressed(slot_index: int)

func _ready() -> void:
	_build_ui()
	if not _armor.update.is_connected(refresh):
		_armor.update.connect(refresh)
	refresh()

func _build_ui() -> void:
	# Fill the parent panel but stay transparent to input; only the slot buttons
	# (MOUSE_FILTER_STOP) capture taps.
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE

	var column := VBoxContainer.new()
	column.name = "ArmorColumn"
	column.add_theme_constant_override("separation", 6)
	add_child(column)
	# Park the column in the top-right corner of the inventory panel, sized to
	# its contents. Tweak the margin/preset here if it overlaps your layout.
	column.set_anchors_and_offsets_preset(Control.PRESET_TOP_RIGHT, Control.PRESET_MODE_MINSIZE, 8)

	var title := Label.new()
	title.text = "Armor"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	column.add_child(title)

	for i in range(4):
		var panel := Panel.new()
		panel.custom_minimum_size = Vector2(96, 88)
		column.add_child(panel)

		var vbox := VBoxContainer.new()
		vbox.set_anchors_preset(Control.PRESET_FULL_RECT)
		vbox.mouse_filter = Control.MOUSE_FILTER_IGNORE
		panel.add_child(vbox)

		var name_label := Label.new()
		name_label.text = SLOT_NAMES[i]
		name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		vbox.add_child(name_label)

		var icon := TextureRect.new()
		icon.custom_minimum_size = Vector2(40, 40)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		vbox.add_child(icon)
		_icons.append(icon)

		var def_label := Label.new()
		def_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		vbox.add_child(def_label)
		_def_labels.append(def_label)

		var spd_label := Label.new()
		spd_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		vbox.add_child(spd_label)
		_spd_labels.append(spd_label)

		# Transparent overlay button so a tap anywhere on the slot registers.
		var button := Button.new()
		button.flat = true
		button.set_anchors_preset(Control.PRESET_FULL_RECT)
		button.mouse_filter = Control.MOUSE_FILTER_STOP
		var idx := i
		button.pressed.connect(func() -> void: armor_slot_pressed.emit(idx))
		panel.add_child(button)
		_buttons.append(button)

## Redraws all 4 slots from the shared ArmorInventory (stats) and the tracked
## Collectibles (icon). Connected to ArmorInventory.update, also called directly.
func refresh(_unused = null) -> void:
	for i in range(4):
		var piece: ArmorPiece = _armor.get_piece(i)
		var equipped := not piece.get_slot().is_empty()
		if equipped:
			var item: Collectibles = _equipped_items[i]
			if item != null and item.icon != null:
				_icons[i].texture = item.icon
				_icons[i].visible = true
			else:
				_icons[i].visible = false
			_def_labels[i].text = "DEF: %d" % piece.get_defense()
			_def_labels[i].visible = true
			# speed_modifier is a fraction (0.05 = +5%, -0.10 = -10%).
			# "%+d" prints an explicit sign for both positive and negative.
			var pct := roundi(piece.get_speed_modifier() * 100.0)
			_spd_labels[i].text = "SPD: %+d%%" % pct
			_spd_labels[i].visible = true
		else:
			_icons[i].visible = false
			_def_labels[i].visible = false
			_spd_labels[i].visible = false

## Equips `item`'s armor piece into its matching slot. Returns the Collectibles
## that was displaced from that slot (or null). The caller is responsible for
## putting any displaced item back into the inventory.
func try_equip(item: Collectibles) -> Collectibles:
	if item == null or not item.is_armor():
		return null
	var piece: ArmorPiece = item.get_armor_piece()
	var idx := piece.get_slot_index()
	if idx < 0 or idx > 3:
		return null
	var displaced: Collectibles = _equipped_items[idx]
	_armor.equip_to_slot(piece, idx)   # updates Rust stats + emits update -> refresh()
	_equipped_items[idx] = item
	refresh()
	return displaced

## Unequips slot `idx`, returning the Collectibles that was there (or null), so
## the caller can return it to the inventory.
func unequip_slot(idx: int) -> Collectibles:
	if idx < 0 or idx > 3:
		return null
	if _armor.is_slot_empty(idx):
		return null
	var removed: Collectibles = _equipped_items[idx]
	_armor.unequip(idx)                # emits update -> refresh()
	_equipped_items[idx] = null
	refresh()
	return removed
