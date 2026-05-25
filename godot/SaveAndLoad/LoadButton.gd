extends Button

var _save_name
var _date
var _image_path
var _seed_game

signal LoadButtonDown(date, saveName, imagePath, seedGame)

func setup_button(data):
	_save_name = data.name
	_date = Time.get_datetime_string_from_unix_time(data.dateTime)
	_image_path = data.imgPath
	_seed_game = data.seed

func get_save_name():
	return _save_name

func get_date():
	return _date

func get_image_path():
	return _image_path

func get_seed_game():
	return _seed_game

func _on_button_down() -> void:
	LoadButtonDown.emit(_date, _save_name, _image_path, _seed_game)
	pass # Replace with function body.
