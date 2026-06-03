use godot::classes::{IResource, Resource};
use godot::prelude::*;

use crate::armor_piece::ArmorPiece;

/// Tracks the four equipped armor pieces (helmet, body, leggings, boots).
///
/// Slots are indexed: 0 = Helmet, 1 = Body, 2 = Leggings, 3 = Boots.
#[derive(GodotClass)]
#[class(base = Resource)]
pub struct ArmorInventory {
    base: Base<Resource>,

    // One entry per slot; a piece whose slot name is empty means nothing equipped.
    equipped: [Gd<ArmorPiece>; 4],
}

#[godot_api]
impl IResource for ArmorInventory {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            equipped: [
                Gd::default(),
                Gd::default(),
                Gd::default(),
                Gd::default(),
            ],
        }
    }
}

#[godot_api]
impl ArmorInventory {
    /// Emitted whenever the equipped set changes (equip / unequip), so UI can redraw.
    #[signal]
    fn update();

    /// Equips `piece`. Any previously-equipped piece in the same slot is silently replaced.
    /// Returns a `Gd` to the old piece (or the default empty `Gd` if the slot was empty).
    #[func]
    pub fn equip(&mut self, piece: Gd<ArmorPiece>) -> Gd<ArmorPiece> {
        let slot_idx = piece.bind().get_slot_index().clamp(0, 3) as usize;
        let old = self.equipped[slot_idx].clone();
        self.equipped[slot_idx] = piece;
        self.signals().update().emit();
        old
    }

    /// Equips `piece` only when its declared slot matches `slot_index` (0–3), e.g.
    /// a helmet can only go in the helmet slot. Returns the previously-equipped
    /// piece (or empty `Gd`). If the piece doesn't belong in that slot, this is a
    /// no-op and returns the default empty `Gd`.
    #[func]
    pub fn equip_to_slot(&mut self, piece: Gd<ArmorPiece>, slot_index: i32) -> Gd<ArmorPiece> {
        if piece.bind().get_slot_index() != slot_index {
            return Gd::default();
        }
        let idx = slot_index.clamp(0, 3) as usize;
        let old = self.equipped[idx].clone();
        self.equipped[idx] = piece;
        self.signals().update().emit();
        old
    }

    /// Unequips whatever is in `slot_index` (0–3) and returns it.
    /// Returns the default empty `Gd` if the slot was already empty.
    #[func]
    pub fn unequip(&mut self, slot_index: i32) -> Gd<ArmorPiece> {
        let idx = slot_index.clamp(0, 3) as usize;
        let old = self.equipped[idx].clone();
        self.equipped[idx] = Gd::default();
        self.signals().update().emit();
        old
    }

    /// Returns the equipped piece in the given slot, or `Gd::default()` if empty.
    #[func]
    pub fn get_piece(&self, slot_index: i32) -> Gd<ArmorPiece> {
        let idx = slot_index.clamp(0, 3) as usize;
        self.equipped[idx].clone()
    }

    /// `true` when the given slot has no equipped piece.
    #[func]
    pub fn is_slot_empty(&self, slot_index: i32) -> bool {
        let idx = slot_index.clamp(0, 3) as usize;
        let piece = &self.equipped[idx];
        // A default-constructed Gd is falsy; an equipped piece has a non-empty slot name.
        piece.bind().get_slot().is_empty()
    }

    /// Total defense from all equipped pieces combined.
    #[func]
    pub fn total_defense(&self) -> i32 {
        self.equipped
            .iter()
            .filter(|p| !p.bind().get_slot().is_empty())
            .map(|p| p.bind().get_defense())
            .sum()
    }

    /// Sum of all equipped pieces' speed modifiers.
    /// Positive values = speed bonus, negative values = slow penalty.
    #[func]
    pub fn total_speed_modifier(&self) -> f32 {
        self.equipped
            .iter()
            .filter(|p| !p.bind().get_slot().is_empty())
            .map(|p| p.bind().get_speed_modifier())
            .sum()
    }

    /// Returns an `Array` of the four equipped `ArmorPiece` entries (indexed 0-3).
    #[func]
    pub fn get_all_pieces(&self) -> Array<Gd<ArmorPiece>> {
        let mut arr = Array::new();
        for piece in &self.equipped {
            arr.push(piece);
        }
        arr
    }
}
