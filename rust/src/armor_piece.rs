use godot::classes::{IResource, Resource};
use godot::prelude::*;

/// The slot an armor piece occupies.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Helmet = 0,
    Body = 1,
    Leggings = 2,
    Boots = 3,
}

impl ArmorSlot {
    /// Returns a human-readable label for the slot.
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            ArmorSlot::Helmet => "Helmet",
            ArmorSlot::Body => "Body",
            ArmorSlot::Leggings => "Leggings",
            ArmorSlot::Boots => "Boots",
        }
    }
}

/// A single armor piece with defensive and mobility stats.
#[derive(GodotClass)]
#[class(base = Resource)]
pub struct ArmorPiece {
    base: Base<Resource>,

    #[export]
    #[var(get = get_slot, set = set_slot)]
    slot: GString,

    #[export]
    #[var(get = get_defense, set = set_defense)]
    defense: i32,

    #[export]
    #[var(get = get_speed_modifier, set = set_speed_modifier)]
    speed_modifier: f32,
}

#[godot_api]
impl IResource for ArmorPiece {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            slot: GString::default(),
            defense: 0,
            speed_modifier: 0.0,
        }
    }
}

#[godot_api]
impl ArmorPiece {
    /// Gets the armor slot as a `GString` (e.g. "Helmet").
    #[func]
    pub fn get_slot(&self) -> GString {
        self.slot.clone()
    }

    /// Sets the armor slot identifier.
    #[func]
    pub fn set_slot(&mut self, slot: GString) {
        self.slot = slot;
    }

    /// Defense value contributed by this piece. Higher is better.
    #[func]
    pub fn get_defense(&self) -> i32 {
        self.defense
    }

    #[func]
    pub fn set_defense(&mut self, value: i32) {
        self.defense = value;
    }

    /// Speed modifier as a percentage (e.g. -0.10 for a 10% slow, +0.05 for a 5% speed boost).
    #[func]
    pub fn get_speed_modifier(&self) -> f32 {
        self.speed_modifier
    }

    #[func]
    pub fn set_speed_modifier(&mut self, value: f32) {
        self.speed_modifier = value;
    }

    /// Convenience: returns the integer index for the slot string, or -1 if unrecognised.
    #[func]
    pub fn get_slot_index(&self) -> i32 {
        match self.slot.to_string().as_str() {
            "Helmet" => 0,
            "Body" => 1,
            "Leggings" => 2,
            "Boots" => 3,
            _ => -1,
        }
    }
}
