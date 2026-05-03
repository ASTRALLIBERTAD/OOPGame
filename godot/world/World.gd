extends Node2dRust

@onready var scene = get_tree()
var peer = ENetMultiplayerPeer.new()
@onready var terrain = $"../Terrain/Terrain1"
@onready var debug_label = $TouchControl/TouchControls/Label # Optional: for displaying stats

var update_interval = 0.5
var time_passed = 0.0

func _ready() -> void:
	$AutoSaveTimer.start()
	GlobalNodeManager.register_terrain(terrain)
	terrain.set_performance_mode(true)

func _process(delta):
	time_passed += delta
	
	if time_passed >= update_interval:
		time_passed = 0.0
		check_queue_health()

func check_queue_health():
	var load_queue = terrain.get_queue_size()
	var unload_queue = terrain.get_unload_queue_size()
	var save_queue = terrain.get_save_queue_size()
	var loaded_chunks = terrain.get_loaded_chunk_count()
	var cached_chunks = terrain.get_cached_chunk_count()
	
	# Display stats if you have a debug label
	if debug_label:
		debug_label.text = "Loaded: %d | Cached: %d | Load Q: %d | Unload Q: %d | Save Q: %d\nFPS: %d" % [
			loaded_chunks, cached_chunks, load_queue, unload_queue, save_queue, Engine.get_frames_per_second()
		]
	
	# Warning if queues are backing up
	if load_queue > 50:
		push_warning("Chunk load queue backing up: %d chunks" % load_queue)
	
	if unload_queue > 30:
		push_warning("Chunk unload queue backing up: %d chunks" % unload_queue)
	
	if save_queue > 40:
		push_warning("Chunk save queue backing up: %d chunks" % save_queue)
	
	# Performance warning if FPS drops
	if Engine.get_frames_per_second() < 30:
		push_warning("Low FPS detected: %d" % Engine.get_frames_per_second())


func _notification(what):
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		terrain.flush_all_queues()  # Save everything before quit
		await get_tree().create_timer(0.5).timeout
		get_tree().quit()


@rpc("any_peer","call_local")
func add_player(pid):
	var plyr = preload("res://Player/players.scn").instantiate() as Rustplayer
	plyr.name = str(pid)
	add_child(plyr)
	
	plyr.set_multiplayer_authority(pid)

func _on_auto_save_timeout() -> void:
	RustSaveManager1.auto_save()
	pass 


func _on_saving_time_timeout() -> void:
	get_tree().paused = false
	RustSaveManager1.set_player_health(0)
	RustSaveManager1.rust_screenshot()
	scene.change_scene_to_file("res://SaveAndLoad/LoadMenu.scn")
	queue_redraw()
	queue_free()

func _on_menu_pressed() -> void:
	%TouchControls.visible = false
	get_tree().paused = true
	%Panel.visible = true
	pass # Replace with function body.

func _on_save_pressed() -> void:
	%TouchControls.visible = false
	%Panel.visible = false
	%CanvasLayer.visible = false
	terrain.flush_all_queues()
	%SavingTime.start()
	pass # Replace with function body.

func _on_back_pressed() -> void:
	%TouchControls.visible = true
	%Panel.visible = false
	get_tree().paused = false
	pass # Replace with function body.


func _on_host_pressed() -> void:
	peer.create_server(5555, 3)
	multiplayer.multiplayer_peer = peer
	%World.broadcast()
	$Broadcaster.start()
	RoomInfo.name = RustSaveManager1.load_game
	var id = multiplayer.get_unique_id()
	$PLAYERS.set_multiplayer_authority(id)
	
	
	multiplayer.peer_connected.connect(
	func(pid):
		print(pid)
		var seeds = terrain.seedser
		$"..".rpc("seed", seeds)
		
		rpc("add_player", pid)
		
		player_node_names.append(str(pid))
	
		)
	multiplayer.peer_disconnected.connect(
		func(pid):
			print(pid)
			get_node(str(pid)).queue_free()
			player_node_names.erase(str(pid))
	)
	
	pass # Replace with function body.
var udp : PacketPeerUDP
var listner: PacketPeerUDP
@export var broadcastPort: int = 8912

var RoomInfo = {"name":"name", "playerCount": 0}
func _on_broadcaster_timeout() -> void:
	var data = JSON.stringify(RoomInfo)
	var packet = data.to_ascii_buffer()
	%World.broadcaster_timeout(packet)
	print(packet)
	pass # Replace with function body.

func cleanUp():
	$Broadcaster.stop()
	if udp != null:
		udp.close()
