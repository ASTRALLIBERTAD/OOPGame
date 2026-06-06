use godot::classes::{ITextureRect, TextureRect};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=TextureRect)]
pub struct HungerDisplay {
    base: Base<TextureRect>,

    #[export(range = (0.0, 2.0))]
    #[var(get, set)]
    hunger: i32,
}

#[godot_api]
impl ITextureRect for HungerDisplay {
    fn init(base: Base<TextureRect>) -> Self {
        Self { base, hunger: 2 }
    }
}

#[godot_api]
impl HungerDisplay {
    #[func]
    pub fn get_hunger(&self) -> i32 {
        self.hunger
    }

    #[func]
    pub fn set_hunger(&mut self, hunger: i32) {
        self.hunger = hunger
    }
}