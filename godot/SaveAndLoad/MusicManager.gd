extends Node

var music_player: AudioStreamPlayer

func _ready() -> void:
	# This ensures the music keeps playing even if you pause the game
	process_mode = Node.PROCESS_MODE_ALWAYS
	
	music_player = AudioStreamPlayer.new()
	add_child(music_player)

func play_music(stream: AudioStream) -> void:
	# If the same track is already playing, do nothing (prevents music restarting on scene reload)
	if music_player.stream == stream and music_player.playing:
		return 
	music_player.stream = stream
	music_player.play()

func stop_music() -> void:
	music_player.stop()
