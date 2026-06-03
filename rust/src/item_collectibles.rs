use godot::classes::{IResource, Resource, Texture2D};
use godot::prelude::*;

use crate::armor_piece::ArmorPiece;

#[derive(GodotClass)]
#[class(base = Resource)]
pub struct Collectibles {
    base: Base<Resource>,

    #[export]
    #[var(get = get_name)]
    name: GString,

    #[export]
    #[var(get = get_amount, set = set_amount)]
    amount: i32,

    #[export]
    icon: Option<Gd<Texture2D>>,

    #[export]
    #[var(get = is_stackable)]
    stackable: bool,

    /// Optional link to the armor stats this item grants when equipped.
    /// `None` for ordinary (non-armor) collectibles.
    #[export]
    armor_piece: Option<Gd<ArmorPiece>>,
}

#[godot_api]
impl IResource for Collectibles {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            name: GString::default(),
            amount: 1,
            icon: None,
            stackable: bool::default(),
            armor_piece: None,
        }
    }
}

#[godot_api]
impl Collectibles {
    #[func]
    pub fn get_name(&self) -> GString {
        self.name.clone()
    }
    #[func]
    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    #[func]
    pub fn set_amount(&mut self, amount: i32) {
        self.amount = amount;
    }

    #[func]
    pub fn is_stackable(&self) -> bool {
        self.stackable
    }

    /// `true` when this collectible is an equippable armor piece.
    ///
    /// Pair with the `#[export]`-generated `get_armor_piece()` getter (which
    /// returns the `ArmorPiece`, or null for non-armor items) — always gate that
    /// call behind `is_armor()` on the GDScript side to avoid a null deref.
    #[func]
    pub fn is_armor(&self) -> bool {
        self.armor_piece.is_some()
    }
}
