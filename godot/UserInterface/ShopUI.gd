extends Control

const ITEMS = [
	"palay",
	"portal",
	"body_armor",
	"boots",
	"helmet",
	"items/sword",
	"items/apple"
]
const SHOP_SIZE = 4
const MAX_STOCK_PER_ITEM = 2
const CLOSE_DISTANCE = 150.0
const RESTOCK_DURATION = 300.0

@onready var items_container: VBoxContainer = $Panel/MarginContainer/VBoxContainer/ItemsContainer
@onready var title_label: Label = $Panel/MarginContainer/VBoxContainer/TitleLabel
@onready var close_btn: Button = $Panel/MarginContainer/VBoxContainer/CloseButton

var current_markup: float = 1.0
var current_trader: Node = null
var player: Node = null
var stock: Array = []

var _trader_stocks: Dictionary = {}
var _restock_timers: Dictionary = {}

func _ready():
	process_mode = Node.PROCESS_MODE_ALWAYS
	EventBus.trade_requested.connect(_on_trade_requested)
	close_btn.pressed.connect(_on_close)
	hide()

func _process(_delta):
	if not visible:
		return
	if not is_instance_valid(current_trader) or not is_instance_valid(player):
		_on_close()
		return
	if player.global_position.distance_to(current_trader.global_position) > CLOSE_DISTANCE:
		EventBus.message.emit("You walked too far from the trader.")
		_on_close()
		return 
		
	_update_restock_label(_get_trader_id())

func _on_trade_requested(markup: float, trader: Node):
	current_markup = markup
	current_trader = trader
	var players = get_tree().get_nodes_in_group("player")
	if players.is_empty():
		return
	player = players[0]
	_load_or_generate_stock()
	_populate_ui()
	if is_instance_valid(current_trader):
		current_trader.call_deferred("set_in_trade", true)
	show()

func _get_trader_id() -> String:
	if current_trader.has_method("get_trader_id"):
		return current_trader.get_trader_id()
	return str(current_trader.get_instance_id())

func _load_or_generate_stock():
	var tid = _get_trader_id()
	if _trader_stocks.has(tid):
		stock = _trader_stocks[tid]
		_update_restock_label(tid)
	else:
		_generate_stock()
		_trader_stocks[tid] = stock
		_start_restock_timer(tid)

func _generate_stock():
	stock = []
	var pool = ITEMS.duplicate()
	pool.shuffle()
	for i in range(min(SHOP_SIZE, pool.size())):
		var path = "res://Collectibles/" + pool[i] + ".res"
		if ResourceLoader.exists(path):
			var item_res = load(path)
			stock.append({
				"resource": item_res,
				"item_id": pool[i],
				"count": MAX_STOCK_PER_ITEM
			})

func _start_restock_timer(tid: String):
	if _restock_timers.has(tid):
		var old: Timer = _restock_timers[tid]
		if is_instance_valid(old):
			old.queue_free()
	var t = Timer.new()
	t.wait_time = RESTOCK_DURATION
	t.one_shot = true
	t.autostart = false
	add_child(t)
	t.timeout.connect(_on_restock_timeout.bind(tid))
	t.start()
	_restock_timers[tid] = t

func _on_restock_timeout(tid: String):
	_trader_stocks.erase(tid)
	_restock_timers.erase(tid)
	if is_instance_valid(current_trader) and visible:
		var current_tid = _get_trader_id()
		if current_tid == tid:
			_generate_stock()
			_trader_stocks[tid] = stock
			_start_restock_timer(tid)
			_populate_ui()
			EventBus.message.emit("The trader has restocked.")

func _get_restock_time_left(tid: String) -> float:
	if _restock_timers.has(tid):
		var t: Timer = _restock_timers[tid]
		if is_instance_valid(t):
			return t.time_left
	return 0.0

func _update_restock_label(tid: String):
	var secs = int(_get_restock_time_left(tid))
	var mins = secs / 60
	var s = secs % 60
	title_label.text = "Tagapamagitan (markup: %d%%) — restock in %d:%02d" % [
		int((current_markup - 1.0) * 100), mins, s
	]

func _populate_ui():
	for child in items_container.get_children():
		child.queue_free()
	
	_update_restock_label(_get_trader_id())
	
	for entry in stock:
		var item = entry["resource"]
		var count = entry["count"]
		if count <= 0:
			continue
		var price = int(item.get_base_price() * current_markup)
		var row = HBoxContainer.new()
		row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var name_label = Label.new()
		name_label.text = item.get_name()
		name_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_label.autowrap_mode = TextServer.AUTOWRAP_OFF
		var count_label = Label.new()
		count_label.text = "x%d" % count
		count_label.custom_minimum_size = Vector2(30, 0)
		var price_label = Label.new()
		price_label.text = "%d₱" % price
		price_label.custom_minimum_size = Vector2(60, 0)
		var buy_btn = Button.new()
		buy_btn.text = "Buy"
		buy_btn.custom_minimum_size = Vector2(60, 0)
		buy_btn.pressed.connect(_on_buy.bind(entry, price))
		row.add_child(name_label)
		row.add_child(count_label)
		row.add_child(price_label)
		row.add_child(buy_btn)
		items_container.add_child(row)

func _on_buy(entry: Dictionary, price: int):
	if entry["count"] <= 0:
		EventBus.message.emit("Out of stock.")
		return
	if not is_instance_valid(player):
		return
	if player.spend_piso(price):
		entry["count"] -= 1
		var scene_path = "res://Collectibles/" + entry["item_id"] + ".scn"
		if ResourceLoader.exists(scene_path):
			var item_scene = load(scene_path)
			var item_spawn = item_scene.instantiate()
			if "global_position" in item_spawn:
				item_spawn.global_position = player.global_position
			get_tree().root.add_child(item_spawn)
			EventBus.message.emit("Bought %s for %d piso." % [entry["resource"].get_name(), price])
		else:
			EventBus.message.emit("Error: Scene file missing for %s." % entry["resource"].get_name())
		_populate_ui()
	else:
		EventBus.message.emit("Not enough piso.")

func _on_close():
	hide()
	if is_instance_valid(current_trader):
		current_trader.call_deferred("set_in_trade", false)
	current_trader = null
	player = null

func get_save_data() -> Dictionary:
	var data = {}
	for tid in _trader_stocks:
		var entries = []
		for entry in _trader_stocks[tid]:
			entries.append({
				"item_id": entry["item_id"],
				"count": entry["count"]
			})
		var time_left = _get_restock_time_left(tid)
		data[tid] = {
			"stock": entries,
			"restock_time_left": time_left
		}
	return data

func load_save_data(data: Dictionary):
	for tid in data:
		var saved = data[tid]
		var entries = []
		for raw in saved["stock"]:
			var path = "res://Collectibles/" + raw["item_id"] + ".res"
			if ResourceLoader.exists(path):
				var item_res = load(path)
				entries.append({
					"resource": item_res,
					"item_id": raw["item_id"],
					"count": raw["count"]
				})
		_trader_stocks[tid] = entries
		var remaining: float = saved.get("restock_time_left", RESTOCK_DURATION)
		var t = Timer.new()
		t.wait_time = max(remaining, 1.0)
		t.one_shot = true
		t.autostart = false
		add_child(t)
		t.timeout.connect(_on_restock_timeout.bind(tid))
		t.start()
		_restock_timers[tid] = t
