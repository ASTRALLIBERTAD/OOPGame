extends Control
@onready var messages_container: VBoxContainer = $Messages

func _ready():
	EventBus.message.connect(add_message)
	messages_container.custom_minimum_size = Vector2(550, 0)

func add_message(text: String) -> void:
	var panel := PanelContainer.new()
	var style_box := StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.4)
	style_box.set_content_margin_all(10)
	style_box.corner_radius_top_left = 8
	style_box.corner_radius_top_right = 8
	style_box.corner_radius_bottom_left = 8
	style_box.corner_radius_bottom_right = 8
	panel.add_theme_stylebox_override("panel", style_box)

	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 24)
	label.autowrap_mode = 3
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	panel.add_child(label)
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	panel.modulate.a = 0.0
	messages_container.add_child(panel)

	var tween_in := create_tween()
	tween_in.tween_property(panel, "modulate:a", 1.0, 0.15)
	await get_tree().create_timer(4.0).timeout
	if not is_instance_valid(panel):
		return
	var tween_out := create_tween()
	tween_out.tween_property(panel, "modulate:a", 0.0, 0.5)
	tween_out.tween_callback(panel.queue_free)
