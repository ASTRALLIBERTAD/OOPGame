extends Area2D

var player_inside: bool = false
var parent: Node = null

func _ready():
	parent = get_parent()
	body_entered.connect(_on_body_entered)
	body_exited.connect(_on_body_exited)

func _process(_delta):
	if player_inside and Input.is_action_just_pressed("interact"):
		parent.on_interact()

func _on_body_entered(body: Node2D) -> void:
	if body.is_in_group("player"):
		player_inside = true
		if parent.is_in_group("trader"):
			EventBus.message.emit("Press I to talk to the OFW")
		elif parent.is_in_group("healer"):
			EventBus.message.emit("Press I to receive a blessing")
		elif parent.is_in_group("civilian"):
			EventBus.message.emit("Press I to talk to the Magsasaka")
		else:
			EventBus.message.emit("Press I to interact")

func _on_body_exited(body: Node2D) -> void:
	if body.is_in_group("player"):
		player_inside = false
