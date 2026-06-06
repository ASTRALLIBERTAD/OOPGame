class_name VirtualJoystick
extends Control

@export_category("Visuals")
@export var pressed_color := Color.GRAY

@export_category("Zones")
@export_range(0, 200, 1) var deadzone_size : float = 10
@export_range(0, 500, 1) var clampzone_size : float = 75

enum Joystick_mode { FIXED, DYNAMIC, FOLLOWING }
@export var joystick_mode := Joystick_mode.FIXED

enum Visibility_mode { ALWAYS, TOUCHSCREEN_ONLY, WHEN_TOUCHED }
@export var visibility_mode := Visibility_mode.ALWAYS

@export_category("Input Actions")
@export var use_input_actions := true
@export var action_left := "ui_left"
@export var action_right := "ui_right"
@export var action_up := "ui_up"
@export var action_down := "ui_down"

@export_category("Audio")
@export var footstep_sound_1: AudioStream
@export var footstep_sound_2: AudioStream
@export_range(0.05, 2.0, 0.01) var step_interval: float = 0.35

var is_pressed := false
var output := Vector2.ZERO

var _touch_index : int = -1
var _movement_audio_player: AudioStreamPlayer
var _use_sound_1: bool = true
var _step_accumulator: float = 0.0

@onready var _base := $Base
@onready var _tip := $Base/Tip

@onready var _base_default_position : Vector2 = _base.position
@onready var _tip_default_position : Vector2 = _tip.position
@onready var _default_color : Color = _tip.modulate

func _ready() -> void:
	_movement_audio_player = AudioStreamPlayer.new()
	add_child(_movement_audio_player)
	
	
	_step_accumulator = step_interval
	
	
	if ProjectSettings.get_setting("input_devices/pointing/emulate_mouse_from_touch"):
		printerr("Virtual Joystick: 'emulate_mouse_from_touch' should be set to False in Project Settings.")
	if not ProjectSettings.get_setting("input_devices/pointing/emulate_touch_from_mouse"):
		printerr("Virtual Joystick: 'emulate_touch_from_mouse' should be set to True in Project Settings.")
	
	# Visibility Checks
	if not DisplayServer.is_touchscreen_available() and visibility_mode == Visibility_mode.TOUCHSCREEN_ONLY:
		hide()
	elif visibility_mode == Visibility_mode.WHEN_TOUCHED:
		hide()

func _process(delta: float) -> void:
	if is_pressed and output != Vector2.ZERO:
		
		var current_speed = output.length()
		_step_accumulator += delta * current_speed
		
		if _step_accumulator >= step_interval:
			_play_footstep()
			_step_accumulator = 0.0 
	else:
		
		_step_accumulator = step_interval
		if _movement_audio_player.playing:
			_movement_audio_player.stop()

func _input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		if event.pressed:
			if _is_point_inside_joystick_area(event.position) and _touch_index == -1:
				if joystick_mode == Joystick_mode.DYNAMIC or joystick_mode == Joystick_mode.FOLLOWING or (joystick_mode == Joystick_mode.FIXED and _is_point_inside_base(event.position)):
					if joystick_mode == Joystick_mode.DYNAMIC or joystick_mode == Joystick_mode.FOLLOWING:
						_move_base(event.position)
					if visibility_mode == Visibility_mode.WHEN_TOUCHED:
						show()
					_touch_index = event.index
					_tip.modulate = pressed_color
					_update_joystick(event.position)
					get_viewport().set_input_as_handled()
		elif event.index == _touch_index:
			_reset()
			if visibility_mode == Visibility_mode.WHEN_TOUCHED:
				hide()
			get_viewport().set_input_as_handled()
			
	elif event is InputEventScreenDrag:
		if event.index == _touch_index:
			_update_joystick(event.position)
			get_viewport().set_input_as_handled()

func _move_base(new_position: Vector2) -> void:
	_base.global_position = new_position - _base.pivot_offset * get_global_transform_with_canvas().get_scale()

func _move_tip(new_position: Vector2) -> void:
	_tip.global_position = new_position - _tip.pivot_offset * _base.get_global_transform_with_canvas().get_scale()

func _is_point_inside_joystick_area(point: Vector2) -> bool:
	var scaled_size = size * get_global_transform_with_canvas().get_scale()
	var x: bool = point.x >= global_position.x and point.x <= global_position.x + scaled_size.x
	var y: bool = point.y >= global_position.y and point.y <= global_position.y + scaled_size.y
	return x and y

func _get_base_radius() -> Vector2:
	return _base.size * _base.get_global_transform_with_canvas().get_scale() / 2

func _is_point_inside_base(point: Vector2) -> bool:
	var _base_radius = _get_base_radius()
	var center : Vector2 = _base.global_position + _base_radius
	var vector : Vector2 = point - center
	return vector.length_squared() <= _base_radius.x * _base_radius.x

func _update_joystick(touch_position: Vector2) -> void:
	var _base_radius = _get_base_radius()
	var center : Vector2 = _base.global_position + _base_radius
	var vector : Vector2 = touch_position - center
	vector = vector.limit_length(clampzone_size)
	
	if joystick_mode == Joystick_mode.FOLLOWING and touch_position.distance_to(center) > clampzone_size:
		_move_base(touch_position - vector)
	
	_move_tip(center + vector)
	
	if vector.length_squared() > deadzone_size * deadzone_size:
		is_pressed = true
		output = (vector - (vector.normalized() * deadzone_size)) / (clampzone_size - deadzone_size)
	else:
		is_pressed = false
		output = Vector2.ZERO
	
	if use_input_actions:
		_update_input_actions()

func _update_input_actions() -> void:
	if output.x >= 0 and Input.is_action_pressed(action_left): Input.action_release(action_left)
	if output.x <= 0 and Input.is_action_pressed(action_right): Input.action_release(action_right)
	if output.y >= 0 and Input.is_action_pressed(action_up): Input.action_release(action_up)
	if output.y <= 0 and Input.is_action_pressed(action_down): Input.action_release(action_down)
	
	if output.x < 0: Input.action_press(action_left, -output.x)
	if output.x > 0: Input.action_press(action_right, output.x)
	if output.y < 0: Input.action_press(action_up, -output.y)
	if output.y > 0: Input.action_press(action_down, output.y)

func _reset() -> void:
	is_pressed = false
	output = Vector2.ZERO
	_touch_index = -1
	_tip.modulate = _default_color
	_base.position = _base_default_position
	_tip.position = _tip_default_position
	
	_step_accumulator = step_interval
	if _movement_audio_player and _movement_audio_player.playing:
		_movement_audio_player.stop()
		
	if use_input_actions:
		for action in [action_left, action_right, action_down, action_up]:
			if Input.is_action_pressed(action):
				Input.action_release(action)

func _play_footstep() -> void:
	if not footstep_sound_1 or not footstep_sound_2:
		return
	
	_movement_audio_player.stream = footstep_sound_1 if _use_sound_1 else footstep_sound_2
	_movement_audio_player.play()
	_use_sound_1 = !_use_sound_1

func _on_step_timer_timeout() -> void:
	pass
