extends CharacterBody2D

var health = 3




func weapon_damage(damage: int):
	if health <= 0:
		health -= damage
		queue_free()
