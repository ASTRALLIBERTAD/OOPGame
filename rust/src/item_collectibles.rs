use godot::classes::{IResource, Resource, Texture2D};
use godot::prelude::*;

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

    #[export]
    base_price: i32,
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
            base_price: 0,
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
}
