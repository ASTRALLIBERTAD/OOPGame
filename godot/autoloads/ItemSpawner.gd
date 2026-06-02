extends Node

func _ready():
	EventBus.item_dropped.connect(spawn_item)
	EventBus.piso_dropped.connect(spawn_piso)
	EventBus.food_ready.connect(_on_food_ready)
	EventBus.balikbayan_box_dropped.connect(_on_balikbayan_box_dropped)

func _on_food_ready(position: Vector2) -> void:
	spawn_item("palay", position)
func _on_balikbayan_box_dropped(position: Vector2) -> void:
	spawn_item("balikbayan_box", position)

func spawn_item(item_id: String, position: Vector2) -> void:
	var path = "res://Collectibles/" + item_id + ".tscn"
	if not ResourceLoader.exists(path):
		push_error("ItemSpawner: scene not found: " + path)
		return
	var instance = load(path).instantiate()
	instance.position = position
	get_tree().current_scene.add_child(instance)

func spawn_piso(amount: int, position: Vector2) -> void:
	var instance = load("res://Collectibles/coin.tscn").instantiate()
	instance.item = load("res://Collectibles/items/coins.tres").duplicate()
	instance.item.amount = amount
	instance.position = position
	get_tree().root.add_child(instance)
