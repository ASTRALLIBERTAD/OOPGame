use godot::classes::{IResource, Resource};
use godot::prelude::*;

use crate::inv_slot::InvSlot;
use crate::item_collectibles::Collectibles;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct Inventory {
    base: Base<Resource>,

    #[export]
    #[var(get = get_slots)]
    slots: Array<Gd<InvSlot>>,
}

#[godot_api]
impl IResource for Inventory {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            slots: Array::new(),
        }
    }
}

#[godot_api]
impl Inventory {
    #[signal]
    pub fn update();

    fn is_slot_allowed(&self, slot_idx: usize, item: &Gd<Collectibles>) -> bool {
        let name = item.bind().get_name().to_string().to_lowercase();
        match slot_idx {
            12 => name.contains("helmet"),
            13 => name.contains("armor") || name.contains("body"),
            14 => name.contains("leggings") || name.contains("pants"),
            15 => name.contains("boots") || name.contains("shoes"),
            _ => slot_idx <= 11,
        }
    }

    #[func]
    pub fn insert(&mut self, item: Gd<Collectibles>, index1: i32, index2: i32) {
        if index1 < 0 {
            let item_name = item.bind().get_name();
            let item_stackable = item.bind().is_stackable();
            let incoming_amount = item.bind().get_amount();

            for (idx, mut slot) in self.slots.iter_shared().enumerate() {
                if !self.is_slot_allowed(idx, &item) {
                    continue;
                }

                let slot_ref = slot.bind_mut();
                let mut existing = slot_ref.get_item();
                let mut existing_ref = existing.bind_mut();

                if item_stackable && existing_ref.get_name() == item_name {
                    let current_amount = existing_ref.get_amount();
                    existing_ref.set_amount(current_amount + incoming_amount);
                    drop(existing_ref);
                    drop(slot_ref);
                    self.signals().update().emit();
                    return;
                }
            }

            for (idx, mut slot) in self.slots.iter_shared().enumerate() {
                if !self.is_slot_allowed(idx, &item) {
                    continue;
                }

                let mut slot_ref = slot.bind_mut();
                let existing = slot_ref.get_item();
                if existing.bind().get_name().is_empty() {
                    drop(existing);
                    slot_ref.set_item(item.clone());
                    drop(slot_ref);
                    self.signals().update().emit();
                    return;
                }
            }
            godot_error!("Inventory is full!");
        } else {
            let idx1 = index1 as usize;
            let idx2 = index2 as usize;

            let slot_a = self.slots.get(idx1).unwrap();
            let slot_b = self.slots.get(idx2).unwrap();

            let item_a = slot_a.bind().get_item();
            let item_b = slot_b.bind().get_item();

            let item_a_empty = item_a.bind().get_name().is_empty();
            let item_b_empty = item_b.bind().get_name().is_empty();

            if !item_a_empty && !self.is_slot_allowed(idx2, &item_a) {
                return;
            }

            if !item_b_empty && !self.is_slot_allowed(idx1, &item_b) {
                return;
            }

            self.slots.set(idx1, &slot_b);
            self.slots.set(idx2, &slot_a);
            self.signals().update().emit();
            godot_print!("Swapped slots {} and {}", index1, index2);
        }
    }

    #[func]
    pub fn get_slots(&self) -> Array<Gd<InvSlot>> {
        self.slots.clone()
    }
}
