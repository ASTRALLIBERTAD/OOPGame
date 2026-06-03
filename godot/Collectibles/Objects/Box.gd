extends Area2D

const ITEM_POOL: Array[String] = [
	"palay",
	"seal_of_reform",
	"black_ledger",
	"coin"
]

@export var drop_count: int = 3
@export var piso_amount: int = 50
@export var scatter_radius: float = 40.0

var _player_inside: bool = false
var _opened: bool = false

func _ready() -> void:
	body_entered.connect(_on_body_entered)
	body_exited.connect(_on_body_exited)

func _process(_delta: float) -> void:
	if _opened:
		return
	if _player_inside and Input.is_action_just_pressed("interact"):
		_open()

func _on_body_entered(body: Node2D) -> void:
	if body.is_in_group("player"):
		_player_inside = true
		EventBus.message.emit("Press I to open the balikbayan box")

func _on_body_exited(body: Node2D) -> void:
	if body.is_in_group("player"):
		_player_inside = false

func _open() -> void:
	_opened = true
	EventBus.message.emit("The balikbayan box burst open!")
	
	$AnimatedSprite2D.speed_scale = 0.5
	$AnimatedSprite2D.play("open")
	
	await $AnimatedSprite2D.animation_finished
	
	var pool := ITEM_POOL.duplicate()
	pool.shuffle()
	var picks := pool.slice(0, min(drop_count, pool.size()))
	
	for i in picks.size():
		var random_angle := randf_range(0.0, TAU)
		var min_distance := scatter_radius * 0.5
		var random_distance := randf_range(min_distance, scatter_radius)
		var offset := Vector2(cos(random_angle), sin(random_angle)) * random_distance
		EventBus.item_dropped.emit(picks[i], global_position + offset)
		
	if piso_amount > 0:
		var piso_angle := randf_range(0.0, TAU)
		var piso_min_dist := scatter_radius * 0.5
		var piso_distance := randf_range(piso_min_dist, scatter_radius)
		var piso_offset := Vector2(cos(piso_angle), sin(piso_angle)) * piso_distance
		EventBus.piso_dropped.emit(piso_amount, global_position + piso_offset)
	
	queue_free()
