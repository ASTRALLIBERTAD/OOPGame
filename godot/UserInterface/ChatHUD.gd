extends Control

# Point this directly to your VBoxContainer
@onready var messages_container: VBoxContainer = $Messages

func add_message(text: String) -> void:
	# 1. Create the individual message background panel
	var panel := PanelContainer.new()
	
	# 2. Style the panel like Minecraft's chat box (Dark, semi-transparent)
	var style_box := StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.4) # Black with 40% opacity
	style_box.set_content_margin_all(6)      # Tiny padding around the text
	panel.add_theme_stylebox_override("panel", style_box)
	
	# 3. Create the text label
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 34) # Large text as requested
	
	# Put the label inside the panel, and the panel into the chat container
	panel.add_child(label)
	
	# 4. Hide the entire panel before adding it to prevent a visual flash
	panel.modulate.a = 0.0
	messages_container.add_child(panel)
	
	# 5. Fade In the panel (which fades the text inside it too!)
	var tween_in := create_tween()
	tween_in.tween_property(panel, "modulate:a", 1.0, 0.15)
	
	# 6. Wait 4 seconds before starting the fade-out
	await get_tree().create_timer(4.0).timeout
	if not is_instance_valid(panel):
		return
		
	# 7. Fade Out & Auto-Delete the panel completely
	var tween_out := create_tween()
	tween_out.tween_property(panel, "modulate:a", 0.0, 0.5)
	tween_out.tween_callback(panel.queue_free)
