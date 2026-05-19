use godot::classes::{IResource, Resource};
use godot::prelude::*;

use crate::inv_slot::InvSlot;
use crate::item_collectibles::Collectibles;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct Inventory {
    base: Base<Resource>,

    #[export]
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
    fn update();

    #[func]
    pub fn insert(&mut self, item: Gd<Collectibles>, index1: i32, index2: i32) {
        if index1 < 0 {
            let item_name = item.bind().get_name();
            let item_stackable = item.bind().is_stackable();

            // stacking loop
            for mut slot in self.slots.iter_shared() {
                let slot_ref = slot.bind_mut();
                let mut existing = slot_ref.get_item();
                let mut existing_ref = existing.bind_mut();

                if item_stackable && existing_ref.get_name() == item_name {
                    let amount = existing_ref.get_amount();
                    existing_ref.set_amount(amount + 1);
                    drop(existing_ref);
                    drop(slot_ref);
                    self.signals().update().emit();
                    return;
                }
            }

            // empty slot loop
            for mut slot in self.slots.iter_shared() {
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
            // Swap two slots
            let a = self.slots.get(index1 as usize).unwrap().clone();
            let b = self.slots.get(index2 as usize).unwrap().clone();
            self.slots.set(index1 as usize, &b);
            self.slots.set(index2 as usize, &a);
            self.signals().update().emit();
            godot_print!("Swapped slots {} and {}", index1, index2);
        }
    }
}
