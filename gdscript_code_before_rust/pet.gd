extends CharacterBody2D

@export var player: Rustplayer # Reference to the player
@export var speed: float = 100.0  # Pet movement speed
@export var stop_threshold: float = 10.0  # Distance at which the pet stops
@export var follow_distance: float = 100.0  # Distance before the pet follows again
@export var follow_delay: float = 0.8  # Delay before the pet starts following again

var _is_following := true  # Tracks whether the pet should follow

func _physics_process(delta):
	if not player:
		return
	var distance_to_player = global_position.distance_to(player.global_position)
	if _is_following:
		if distance_to_player > stop_threshold:
			$AnimatedSprite2D.play("run")
			_flip_sprite()
			_move_toward_player()
		else:
			$AnimatedSprite2D.play("idle")
			_stop_moving()
			_is_following = false  # Stop following when close
	elif distance_to_player > follow_distance:
		_start_follow_delay()  # Wait before resuming movement

func _move_toward_player():
	var direction = (player.global_position - global_position).normalized()
	velocity = direction * speed
	move_and_slide()

func _stop_moving():
	velocity = Vector2.ZERO
	move_and_slide()

func _start_follow_delay():
	_is_following = true
	await get_tree().create_timer(follow_delay).timeout  # Smooth delay before following

func _flip_sprite():
	if player.global_position.x < global_position.x:
		$AnimatedSprite2D.flip_h = true  # Face left
	else:
		$AnimatedSprite2D.flip_h = false  # Face right
