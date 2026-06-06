extends Control

const FIRST_RUN_KEY = "first_run_done"

func _ready() -> void:
	OS.request_permissions()
	$Transition/ColorRect.visible = false
	
	# Show firewall dialog on first run (Linux/Windows/Mac only)
	if not OS.get_name() in ["Android", "iOS"]:
		if not ProjectSettings.get_setting(FIRST_RUN_KEY, false):
			$FirewallDialog.popup_centered()

func _on_firewall_dialog_confirmed() -> void:
	ProjectSettings.set_setting(FIRST_RUN_KEY, true)
	ProjectSettings.save()

func _on_play_pressed() -> void:
	$AudioStreamPlayer.play()
	$Transition/ColorRect.visible = true
	$Transition.play("fade_out")

func _on_settings_pressed() -> void:
	$AudioStreamPlayer.play()
	await $AudioStreamPlayer.finished
	get_tree().change_scene_to_file("res://UserInterface/SettingMenu.scn")

func _on_exit_pressed() -> void:	
	$AudioStreamPlayer.play()
	await $AudioStreamPlayer.finished
	get_tree().quit()
	
func _on_transition_animation_finished(_anim_name: StringName) -> void:
	get_tree().change_scene_to_file("res://SaveAndLoad/LoadMenu.scn")
