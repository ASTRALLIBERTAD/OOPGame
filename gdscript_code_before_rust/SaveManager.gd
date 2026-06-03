extends Node

var _base_path = "user://"
var _load_game : String
var _player_node: Rustplayer

func world_exist(world_name: String) -> bool:
	var world_file = _base_path + world_name + "/" + world_name +".dat"
	return FileAccess.file_exists(world_file)



func save_game(name):

	var dir = DirAccess.open(_base_path)
	if !dir.dir_exists("games"):
		dir.make_dir("games")
	dir = DirAccess.open(_base_path + "games")

	if !dir.dir_exists(name):
		dir.make_dir(name)
	var file = FileAccess.open(_base_path + "games/" + name + "/" + name + ".dat", FileAccess.WRITE)

	if _player_node != null:
		file.store_var(_player_node.position.x)
		file.store_var(_player_node.position.y)
	else:
		print("Error: player_node is not set.")

	file.close()
	print("Game saved successfully.")

	var SaveGameInfo = {
		"name" : name,
		"imgPath" : _base_path + "games/" + name + "/" + name + ".png",
		"dateTime" : Time.get_unix_time_from_system()
	}
	var SaveGameJson = JSON.stringify(SaveGameInfo)

	var SaveGameFile = FileAccess.open(_base_path + "games/" + name + "/" + name + "_saveGame.json", FileAccess.WRITE)
	SaveGameFile.store_string(SaveGameJson)

	var screenshot = get_viewport().get_texture().get_image()
	screenshot.save_png(_base_path + "games/" + name + "/" + name + ".png")
	file.close()


func _optimize_autosave(name):
	var dir = DirAccess.open(_base_path)
	if !dir.dir_exists("games"):
		dir.make_dir("games")
	dir = DirAccess.open(_base_path + "games")

	if !dir.dir_exists(name):
		dir.make_dir(name)

	var file = FileAccess.open(_base_path + "games/" + name + "/" + name + ".dat", FileAccess.WRITE)

	if _player_node != null:
		file.store_var(_player_node.position.x)
		file.store_var(_player_node.position.y)
	else:
		print("Error: player_node is not set.")

	file.close()
	print("Game saved successfully.")
	pass

func _load_game_by_name(name):
	_load_game = name
	var file_path = _base_path + "games/" + name + "/" + name + ".dat"

	if FileAccess.file_exists(file_path):
		var file = FileAccess.open(file_path, FileAccess.READ)

		if _player_node != null:
			_player_node.position.x = file.get_var()
			_player_node.position.y = file.get_var()
			print("Game loaded successfully.")
		else:
			print("Error: player_node is not set.")

		file.close()
	else:
		print("No data file found.")



func _delete_save(name):
	var dir_path = _base_path + "games/" + name
	var dir = DirAccess.open(dir_path)
	if dir.dir_exists(dir_path):
		var files = dir.get_files()
		for file in files:
			dir.remove(dir_path + "/" + file)
		dir.remove(dir_path)
		print("Save game '" + name + "' deleted successfully.")
	else:
		print("Save game '" + name + "' not found.")

func get_loaded_world_name():
	return _load_game

func auto_save():
	var world_name = get_loaded_world_name()
	print(world_name)
	if world_name != "":
		_optimize_autosave(world_name)
	else :
		print("no world")

func get_player_node() -> Rustplayer:
	return _player_node

func set_player_node(value: Rustplayer) -> void:
	_player_node = value

func get_base_path() -> String:
	return _base_path
