extends Node

signal message(text: String)
signal bribe_requested(bribe_type: String, amount: int, mob: Node)
signal bribe_resolved(accepted: bool)
signal piso_changed(new_total: int)

signal article_published(intel_count: int)
signal boss_defeated()
signal item_dropped(item_id: String, position: Vector2)
signal piso_dropped(amount: int, position: Vector2)

signal food_ready(position: Vector2)
signal enemy_killed_near_farmer(farmer)
signal civilian_killed()

signal balikbayan_box_dropped(position: Vector2)

signal trade_requested(markup: float, trader: Node)

signal press_blackout(duration: float)

func _ready():
	if false:
		message.emit("")
		bribe_requested.emit("", 0, null)
		bribe_resolved.emit(false)
		piso_changed.emit(0)
		civilian_killed.emit()
		article_published.emit(0)
		boss_defeated.emit()
		item_dropped.emit("", Vector2.ZERO)
		piso_dropped.emit(0, Vector2.ZERO)
		
		food_ready.emit(Vector2.ZERO)
		enemy_killed_near_farmer.emit(null)
		civilian_killed.emit()
		
		balikbayan_box_dropped.emit(Vector2.ZERO)
		
		trade_requested.emit(0.0, null)
		
		press_blackout.emit(0.0)
