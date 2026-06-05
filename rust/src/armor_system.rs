use godot::classes::{IResource, Resource};
use godot::prelude::*;

use crate::inv_slot::InvSlot;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct ArmorSystem {
    base: Base<Resource>,
}

#[godot_api]
impl IResource for ArmorSystem {
    fn init(base: Base<Resource>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl ArmorSystem {
    #[func]
    pub fn total_defense(&self, slots: Array<Gd<InvSlot>>) -> i32 {
        (12..=15usize)
            .filter_map(|i| slots.get(i))
            .filter(|slot| !slot.bind().get_item().bind().get_name().is_empty())
            .map(|slot| slot.bind().get_item().bind().get_defense())
            .sum()
    }

    #[func]
    pub fn total_speed_modifier(&self, slots: Array<Gd<InvSlot>>) -> f32 {
        (12..=15usize)
            .filter_map(|i| slots.get(i))
            .filter(|slot| !slot.bind().get_item().bind().get_name().is_empty())
            .map(|slot| slot.bind().get_item().bind().get_speed_modifier())
            .sum()
    }

    #[func]
    pub fn is_slot_empty(&self, slots: Array<Gd<InvSlot>>, slot_index: i32) -> bool {
        let idx = (slot_index + 12) as usize;
        match slots.get(idx) {
            Some(slot) => slot.bind().get_item().bind().get_name().is_empty(),
            None => true,
        }
    }

    #[func]
    pub fn get_piece(
        &self,
        slots: Array<Gd<InvSlot>>,
        slot_index: i32,
    ) -> Gd<crate::item_collectibles::Collectibles> {
        let idx = (slot_index + 12) as usize;
        match slots.get(idx) {
            Some(slot) => slot.bind().get_item(),
            None => Gd::default(),
        }
    }

    #[func]
    pub fn damage_durability(&self, slots: Array<Gd<InvSlot>>, amount: i32) {
        for i in 12..=15usize {
            let Some(mut slot) = slots.get(i) else {
                continue;
            };
            let slot_ref = slot.bind_mut();
            let mut item = slot_ref.get_item();
            let mut item_ref = item.bind_mut();

            if item_ref.get_name().is_empty() {
                continue;
            }

            let new_durability = item_ref.get_durability() - amount;
            if new_durability <= 0 {
                drop(item_ref);
                drop(item);
                drop(slot_ref);
                slot.bind_mut().clear_item();
                godot_print!("Armor in slot {} broke!", i);
            } else {
                item_ref.set_durability(new_durability);
            }
        }
    }
}
