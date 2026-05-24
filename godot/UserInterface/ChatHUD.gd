extends Control

@onready var messages_container: VBoxContainer = $Messages

func _ready():
	EventBus.message.connect(add_message)

func add_message(text: String) -> void:
	var panel := PanelContainer.new()
	var style_box := StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.4)
	style_box.set_content_margin_all(6)
	panel.add_theme_stylebox_override("panel", style_box)

	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 34)

	panel.add_child(label)
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
