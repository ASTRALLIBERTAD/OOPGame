use godot::classes::{IResource, Resource, Texture2D};
use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Default, Clone)]
#[godot(via = GString)]
pub enum ItemType {
    #[default]
    Generic,
    Helmet,
    BodyArmor,
    Leggings,
    Boots,
    Weapon,
    Consumable,
}

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

    #[export]
    item_type: ItemType,

    #[export]
    #[export_group(name = "Combat Stats")]
    #[var(get = get_defense)]
    defense: i32,

    #[export]
    #[export_group(name = "Combat Stats")]
    attack_power: i32,

    #[export]
    #[export_group(name = "Combat Stats")]
    #[var(get = get_speed_modifier)]
    speed_modifier: f32,

    #[export]
    #[export_group(name = "Equipment Condition")]
    #[var(get = get_durability, set = set_durability)]
    durability: i32,
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
            item_type: ItemType::Generic,
            defense: 0,
            attack_power: 0,
            speed_modifier: 0.0,
            durability: 0,
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

    #[func]
    pub fn get_defense(&self) -> i32 {
        self.defense
    }

    #[func]
    pub fn get_speed_modifier(&self) -> f32 {
        self.speed_modifier
    }

    #[func]
    pub fn get_durability(&self) -> i32 {
        self.durability
    }

    #[func]
    pub fn set_durability(&mut self, value: i32) {
        self.durability = value;
    }
}
