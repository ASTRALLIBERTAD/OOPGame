extends Control

@onready var bribe_label = $Panel/Label
@onready var accept_btn = $Panel/AcceptButton
@onready var reject_btn = $Panel/RejectButton

var _current_mob: Node = null
var _bribe_type: String = ""
var _amount: int = 0

func _ready():
	process_mode = Node.PROCESS_MODE_ALWAYS
	EventBus.bribe_requested.connect(_on_bribe_requested)
	accept_btn.pressed.connect(_on_accept)
	reject_btn.pressed.connect(_on_reject)
	hide()

func _on_bribe_requested(bribe_type: String, amount: int, mob: Node):
	_bribe_type = bribe_type
	_amount = amount
	_current_mob = mob
	var messages = {
		"toll": "Pay %d piso to pass?",
		"boss": "Accept the deal? (%d piso)",
		"broker": "Settle quietly? (%d piso)"
	}
	bribe_label.text = messages[bribe_type] % amount
	get_tree().paused = true
	show()

func _on_accept():
	hide()
	get_tree().paused = false
	EventBus.bribe_resolved.emit(true)
	var player = get_tree().get_nodes_in_group("player")[0]
	if not player.spend_piso(_amount):
		_reject_mob()
		return
	match _bribe_type:
		"toll":
			_current_mob.on_toll_paid()
		"boss":
			_current_mob.on_bribe_accepted()
		"broker":
			_current_mob.on_player_bribe_accepted(_amount)
			player.apply_indebted()

func _on_reject():
	hide()
	get_tree().paused = false
	EventBus.bribe_resolved.emit(false)
	_reject_mob()

func _reject_mob():
	match _bribe_type:
		"toll":
			_current_mob.on_toll_refused()
		"boss":
			_current_mob.on_bribe_rejected()
		"broker":
			_current_mob.on_player_bribe_rejected()
