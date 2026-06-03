extends Control

var _world_name: String
@onready var _newgame: = RustSaveManager1

func _on_timer_timeout() -> void:
	var world_name = %WorldNameInput.text
	RustSaveManager1.set_current_world_name(world_name)
	var game_seed = %Seed.text.strip_edges()
	print(world_name)

	if world_name == "":
		print("ERROR")
		return
	#if SaveManager.world_exist(world_name):
		#print("world name already exist")
		#return

	var world = preload("res://world/World.scn").instantiate() #res://World.tscn
	if !get_tree() == null:
		if game_seed.is_valid_int():
			RustSaveManager1.world_seed = int(game_seed)
			var uop = world.get_node("Terrain/Terrain1") as Terrain1
			uop.world_seed = RustSaveManager1.world_seed
			print(uop.world_seed)
			get_tree().root.add_child(world)
			queue_free()
		elif game_seed == "":
			var lp: = RandomNumberGenerator.new()
			var ti = hash(lp)
			var yoj = clampi(ti, -2147483648, 2147483647)
			RustSaveManager1.world_seed = yoj
			print(yoj)
			var up = world.get_node("Terrain/Terrain1") as Terrain1
			up.world_seed = yoj
			print(up.world_seed)
			get_tree().root.add_child(world)
			queue_free()
		else:
			var t: = hash(game_seed)
			RustSaveManager1.world_seed = clampi(t, -2147483648, 2147483647)

			var upl = world.get_node("Terrain/Terrain1") as Terrain1
			upl.world_seed = RustSaveManager1.world_seed
			print(upl.world_seed)
			get_tree().root.add_child(world)
			queue_free()

		RustSaveManager1.save_game_rust(world_name)
		RustSaveManager1.save_world()
		RustSaveManager1.set_player_health(20)
	else:
		print("failed to  save a new game")

	_newgame.save_game_rust(world_name)

func _on_backbutton_pressed() -> void:
	get_tree().change_scene_to_file("res://SaveAndLoad/LoadMenu.scn")

func _on_back_pressed() -> void:
	get_tree().change_scene_to_file("res://SaveAndLoad/LoadMenu.scn")

func _on_playbuton_pressed() -> void:
	$Timer.start()
