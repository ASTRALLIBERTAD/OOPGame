extends Node

func _ready():
	EventBus.civilian_killed.connect(_on_civilian_killed)
	get_tree().node_added.connect(_on_node_added)
	get_tree().root.child_entered_tree.connect(_on_scene_changed)

func _on_scene_changed(node: Node):
	if node.scene_file_path == "res://world/World.scn":
		await get_tree().process_frame
		for enemy in get_tree().get_nodes_in_group("enemy"):
			if not enemy.tree_exiting.is_connected(_on_enemy_removing):
				enemy.tree_exiting.connect(_on_enemy_removing.bind(enemy))
		print("World loaded — enemies connected: ", get_tree().get_nodes_in_group("enemy").size())

func _on_node_added(node):
	if node.is_in_group("enemy"):
		
		node.tree_exiting.connect(_on_enemy_removing.bind(node))

func _on_enemy_removing(enemy):
	if enemy == null:
		return
		
	var epos: Vector2 = enemy.global_position
	
	var farmers = get_tree().get_nodes_in_group("civilian")
	for farmer in farmers:
		if is_instance_valid(farmer) and farmer.has_method("on_enemy_killed_nearby"):
			if farmer.global_position.distance_to(epos) <= 300.0:
				farmer.on_enemy_killed_nearby()

func _on_civilian_killed():
	var students = get_tree().get_nodes_in_group("student")
	for student in students:
		if is_instance_valid(student) and student.has_method("on_civilian_killed"):
			student.on_civilian_killed()
