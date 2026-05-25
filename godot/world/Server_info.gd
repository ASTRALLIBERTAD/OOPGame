extends HBoxContainer

signal joinGame(ip)
# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.



func _on_button_button_down():
	joinGame.emit($Ip.text)
	var ip = $Ip.text
	var port = $PlayerCount.text
	var peer = ENetMultiplayerPeer.new()
	var error = peer.create_client(ip, 5555)
	
	if error == OK:
		multiplayer.multiplayer_peer = peer
		print("Connecting ", ip ) 
		get_tree().change_scene_to_file("res://world/World.scn")
	else:
		print("Failed to create client. Error code:", error)
	pass # Replace with function body.
