extends Node2dRust

@onready var scene = get_tree()
var peer: ENetMultiplayerPeer
@onready var terrain = $"../Terrain/Terrain1"
@onready var debug_label = $PauseMenu/CanvasLayer/Label # Optional: for displaying stats

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


func _on_saving_time_timeout() -> void:
	get_tree().paused = false
	
	RustSaveManager1.rust_screenshot()
	scene.change_scene_to_file("res://SaveAndLoad/LoadMenu.scn")
	queue_redraw()
	queue_free()

func _on_menu_pressed() -> void:
	var player_menus = %PLAYERS.get_node("Control/CanvasLayer") as CanvasLayer 
	player_menus.visible = false
	var player_control = %PLAYERS.get_node("Control/TouchControls") as CanvasLayer
	player_control.visible = false
	get_tree().paused = true
	%Panel.visible = true

func _on_save_pressed() -> void:
	var player_menus = %PLAYERS.get_node("Control/CanvasLayer") as CanvasLayer 
	player_menus.visible = false
	var player_control = %PLAYERS.get_node("Control/TouchControls") as CanvasLayer 
	player_control.visible = false
	%Panel.visible = false
	%CanvasLayer.visible = false
	$AutoSaveTimer.stop() 
	terrain.flush_all_queues()
	%SavingTime.start()


func _on_back_pressed() -> void:
	%TouchControls.visible = true
	%Panel.visible = false
	get_tree().paused = false


func _on_host_pressed() -> void:
	if multiplayer.has_multiplayer_peer():
		multiplayer.multiplayer_peer = null
	
	for conn in multiplayer.peer_connected.get_connections():
		multiplayer.peer_connected.disconnect(conn.callable)
	
	for conn in multiplayer.peer_disconnected.get_connections():
		multiplayer.peer_disconnected.disconnect(conn.callable)
	
	peer = ENetMultiplayerPeer.new()
	var err = peer.create_server(5555, 3)
	if err != OK:
		push_error("Failed to create server: %s" % err)
		return
	
	multiplayer.multiplayer_peer = peer
	%World.broadcast()
	$Broadcaster.start()
	
	RoomInfo.name = RustSaveManager1.get_current_world_name()
	
	var id = multiplayer.get_unique_id()
	$PLAYERS.set_multiplayer_authority(id)
	
	multiplayer.peer_connected.connect(func(pid):
		print(pid)
		var seeds = terrain.world_seed
		$"..".rpc("seed", seeds)
		rpc("add_player", pid)
		player_node_names.append(str(pid))
	)
	
	multiplayer.peer_disconnected.connect(func(pid):
		print(pid)
		get_node(str(pid)).queue_free()
		player_node_names.erase(str(pid))
	)

var udp : PacketPeerUDP
var listner: PacketPeerUDP
@export var broadcastPort: int = 8912

var RoomInfo = {"name":"name", "playerCount": 0}
func _on_broadcaster_timeout() -> void:
	var data = JSON.stringify(RoomInfo)
	var packet = data.to_ascii_buffer()
	%World.broadcaster_timeout(packet)
	print(packet)

func cleanUp():
	$Broadcaster.stop()
	if udp != null:
		udp.close()

# This function will handle changing the scene back to the Main Menu
func _on_back_button_pressed() -> void:
	# Change this file path string if your main menu scene lives somewhere else!
	var main_menu_path = "res://UserInterface/MainMenu.scn" 
	
	if ResourceLoader.exists(main_menu_path):
		get_tree().change_scene_to_file(main_menu_path)
	else:
		print("Error: Could not find MainMenu scene file at: ", main_menu_path)
