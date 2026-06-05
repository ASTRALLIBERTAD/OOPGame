extends Panel
@onready var _item_visual: Sprite2D = $CenterContainer/Panel/item
@onready var _amount_text: Label = $CenterContainer/Panel/Label

signal out

func _ready() -> void:
	out.emit()

func update(slot: InvSlot):
	if !slot.item.name:
		_item_visual.visible = false
		_amount_text.visible = false
		_amount_text.text = ""
	else:
		_item_visual.visible = true
		_item_visual.texture = slot.item.icon
		var amt = slot.item.amount
		if amt > 1:
			_amount_text.text = str(amt)
			_amount_text.visible = true
		else:
			_amount_text.text = ""
			_amount_text.visible = false

func _redraw(slot: InvSlot):
	pass
	#if slot.get_item().amount <= 1:
		#
		#_amount_text.visible = false
		#var t = slot.item.amount
		#_amount_text.text = str(t)
	

func _on_button_pressed() -> void:
	out.emit()
	pass # Replace with function body.
