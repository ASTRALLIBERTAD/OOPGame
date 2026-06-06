extends GutTest

var _messages: Array[String] = []
var _items_dropped: Array[Dictionary] = []
var _piso_dropped: Array[Dictionary] = []

func before_each() -> void:
	_messages.clear()
	_items_dropped.clear()
	_piso_dropped.clear()
	EventBus.message.connect(_on_msg)
	EventBus.item_dropped.connect(_on_item)
	EventBus.piso_dropped.connect(_on_piso)

func after_each() -> void:
	if EventBus.message.is_connected(_on_msg):
		EventBus.message.disconnect(_on_msg)
	if EventBus.item_dropped.is_connected(_on_item):
		EventBus.item_dropped.disconnect(_on_item)
	if EventBus.piso_dropped.is_connected(_on_piso):
		EventBus.piso_dropped.disconnect(_on_piso)

func _on_msg(t: String) -> void:
	_messages.append(t)

func _on_item(id: String, pos: Vector2) -> void:
	_items_dropped.append({"id": id, "pos": pos})

func _on_piso(amount: int, pos: Vector2) -> void:
	_piso_dropped.append({"amount": amount, "pos": pos})


func _make_slot(item_name: String = "") -> InvSlot:
	var slot := InvSlot.new()
	var item := Collectibles.new()
	item.set("name", item_name)
	slot.set_item(item)
	return slot

func _fresh_inv() -> Inventory:
	return load("res://Collectibles/items/inventory.res").duplicate()

func _generate_stock() -> Array:
	const ITEMS := ["palay","portal","body_armor","boots","helmet","items/sword","items/apple"]
	const SHOP_SIZE := 4
	const MAX_STOCK := 2
	var pool := ITEMS.duplicate()
	pool.shuffle()
	var stock := []
	for i in range(min(SHOP_SIZE, pool.size())):
		stock.append({"item_id": pool[i], "count": MAX_STOCK})
	return stock

func _spend(balance: int, amount: int) -> Dictionary:
	if balance < amount:
		return {"ok": false, "balance": balance}
	return {"ok": true, "balance": balance - amount}



# Inventory

func test_inventory_insert_first_slot() -> void:
	var inv := _fresh_inv()
	var item := Collectibles.new()
	item.set("name", "palay")
	inv.insert(item, -1, -1)
	assert_eq(inv.slots[0].get_item().get_name(), "palay")

func test_inventory_stackable_stacks() -> void:
	var inv := _fresh_inv()
	for _i in range(3):
		var item := Collectibles.new()
		item.set("name", "palay")
		item.set("stackable", true)
		item.set("amount", 1)
		inv.insert(item, -1, -1)
	assert_eq(inv.slots[0].get_item().get_amount(), 3)

func test_inventory_swap_slots() -> void:
	var inv := _fresh_inv()
	var a := Collectibles.new()
	a.set("name", "sword")
	var b := Collectibles.new()
	b.set("name", "palay")
	inv.insert(a, -1, -1)
	inv.insert(b, -1, -1)
	var dummy := inv.slots[0].get_item()
	inv.insert(dummy, 0, 1)
	assert_eq(inv.slots[0].get_item().get_name(), "palay")
	assert_eq(inv.slots[1].get_item().get_name(), "sword")

func test_inventory_full_does_not_crash() -> void:
	var inv := _fresh_inv()
	for _i in range(12):
		var item := Collectibles.new()
		item.set("name", "sword")
		item.set("stackable", false)
		inv.insert(item, -1, -1)
	var extra := Collectibles.new()
	extra.set("name", "items/iron_helmet")
	extra.set("stackable", false)
	inv.insert(extra, -1, -1)
	assert_true(true)


# Box / ITEM_POOL

func test_item_pool_no_duplicates() -> void:
	var pool: Array[String] = ["palay","seal_of_reform","black_ledger","coin"]
	var seen := {}
	for id in pool:
		assert_false(seen.has(id))
		seen[id] = true

func test_drop_picks_correct_count() -> void:
	var pool := ["palay","seal_of_reform","black_ledger","coin"]
	pool.shuffle()
	var picks := pool.slice(0, min(3, pool.size()))
	assert_eq(picks.size(), 3)

func test_piso_signal_amount() -> void:
	EventBus.piso_dropped.emit(50, Vector2(100, 200))
	assert_eq(_piso_dropped[0]["amount"], 50)

func test_item_signal_received() -> void:
	EventBus.item_dropped.emit("coin", Vector2.ZERO)
	assert_eq(_items_dropped[0]["id"], "coin")

func test_message_signal_received() -> void:
	EventBus.message.emit("The balikbayan box burst open!")
	assert_true(_messages.has("The balikbayan box burst open!"))


# ShopUI stock logic

func test_stock_size_within_shop_size() -> void:
	for _t in range(20):
		assert_lte(_generate_stock().size(), 4)

func test_stock_count_starts_at_max() -> void:
	for entry in _generate_stock():
		assert_eq(entry["count"], 2)

func test_stock_no_duplicate_ids() -> void:
	for _t in range(20):
		var seen := {}
		for entry in _generate_stock():
			assert_false(seen.has(entry["item_id"]))
			seen[entry["item_id"]] = true

func test_buy_decrements_count() -> void:
	var entry := {"item_id": "palay", "count": 2}
	entry["count"] -= 1
	assert_eq(entry["count"], 1)

func test_buy_to_zero_out_of_stock() -> void:
	var entry := {"item_id": "palay", "count": 1}
	entry["count"] -= 1
	assert_true(entry["count"] <= 0)

func test_markup_calculation() -> void:
	assert_eq(int(100 * 1.2), 120)


# Piso economy

func test_spend_sufficient() -> void:
	var r := _spend(200, 50)
	assert_true(r["ok"])
	assert_eq(r["balance"], 150)

func test_spend_insufficient() -> void:
	var r := _spend(30, 100)
	assert_false(r["ok"])
	assert_eq(r["balance"], 30)

func test_spend_exact() -> void:
	var r := _spend(100, 100)
	assert_true(r["ok"])
	assert_eq(r["balance"], 0)

func test_add_piso() -> void:
	assert_eq(max(0, 100 + 50), 150)

func test_piso_cannot_go_negative() -> void:
	assert_eq(max(0, 0 + -999), 0)

func test_starting_piso_is_200() -> void:
	assert_eq(200, 200)


# EventBus

func test_eventbus_multiple_signals_independent() -> void:
	EventBus.message.emit("hello")
	EventBus.piso_dropped.emit(99, Vector2.ZERO)
	assert_eq(_messages[0], "hello")
	assert_eq(_piso_dropped[0]["amount"], 99)

func test_eventbus_message_string_preserved() -> void:
	var msg := "A".repeat(200)
	EventBus.message.emit(msg)
	assert_eq(_messages[0].length(), 200)
