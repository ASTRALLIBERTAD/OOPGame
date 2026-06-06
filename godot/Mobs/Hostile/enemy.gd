extends CharacterBody2D

# 1. Load the sound file directly into the code
const DEATH_SOUND_ASSET = preload("res://Assets/Audio/enemy_dead.wav")

var _health = 3
var is_dying = false

func get_health() -> int:
	return _health

func set_health(value: int) -> void:
	_health = clampi(value, 0, 9999)

func weapon_damage(damage: int):
	if is_dying: 
		return 
		
	_health -= damage
	
	if _health <= 0:
		is_dying = true
		
		# Hide the enemy visuals instantly
		$AnimatedSprite2D.hide() 
		$CollisionShape2D.set_deferred("disabled", true) 
		$Area2D/CollisionShape2D.set_deferred("disabled", true)
		
		# 2. Create a clean, global audio player so distance/parents don't mute it
		var audio_player = AudioStreamPlayer.new()
		audio_player.stream = DEATH_SOUND_ASSET
		
		# Add it to the active game scene, not the dying enemy
		get_tree().current_scene.add_child(audio_player)
		audio_player.play()
		
		# 3. Tell the audio player to delete itself when done, and free the enemy immediately
		audio_player.finished.connect(audio_player.queue_free)
		queue_free()
