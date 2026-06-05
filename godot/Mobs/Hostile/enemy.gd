extends CharacterBody2D

var _health = 3

func get_health() -> int:
	return _health

func set_health(value: int) -> void:
	_health = clampi(value, 0, 9999)

func weapon_damage(damage: int):
	if _health <= 0:
		_health -= damage
		queue_free()
