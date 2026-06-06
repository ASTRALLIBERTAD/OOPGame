use godot::classes::{HBoxContainer, IHBoxContainer, Texture2D};
use godot::prelude::*;

use crate::hunger_display::HungerDisplay;

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct HungerSystem {
    base: Base<HBoxContainer>,

    #[export]
    full_hunger: OnEditor<Gd<Texture2D>>,

    #[export]
    half_hunger: OnEditor<Gd<Texture2D>>,

    #[export]
    empty_hunger: OnEditor<Gd<Texture2D>>,

    hunger_list: Vec<Gd<HungerDisplay>>,

    #[export(range = (0.0, 20.0))]
    #[var(get = get_current_hunger, set = set_current_hunger)]
    current_hunger: i32,
}

#[godot_api]
impl IHBoxContainer for HungerSystem {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,
            full_hunger: OnEditor::default(),
            half_hunger: OnEditor::default(),
            empty_hunger: OnEditor::default(),
            hunger_list: Vec::new(),
            current_hunger: i32::default(),
        }
    }

    fn ready(&mut self) {
        let hunger_parents = self.base_mut().get_children();
        for i in hunger_parents.iter_shared() {
            if let Ok(texture_rect) = i.try_cast::<HungerDisplay>() {
                self.hunger_list.push(texture_rect);
            }
        }
        godot_print!("Hunger list length: {:?}", self.hunger_list);
        godot_print!("Hunger UI is ready!");
    }
}

#[godot_api]
impl HungerSystem {
    pub fn set_hunger_display(&mut self, hunger: i32) {
        let full_hunger: &Gd<Texture2D> = self.full_hunger.to_godot();
        let half_hunger: &Gd<Texture2D> = self.half_hunger.to_godot();
        let empty_hunger: &Gd<Texture2D> = self.empty_hunger.to_godot();

        self.current_hunger = hunger.clamp(0, 20);
        godot_print!("current hunger: {:?}", self.current_hunger);

        let mut remaining = self.current_hunger;

        for icon in &mut self.hunger_list {
            let mut hunger_node = icon.bind_mut();

            if remaining >= 2 {
                hunger_node.set_hunger(2);
                hunger_node.base_mut().set_texture(&full_hunger.clone());
                remaining -= 2;
            } else if remaining == 1 {
                hunger_node.set_hunger(1);
                hunger_node.base_mut().set_texture(&half_hunger.clone());
                remaining -= 1;
            } else {
                hunger_node.set_hunger(0);
                hunger_node.base_mut().set_texture(&empty_hunger.clone());
            }
        }
    }

    #[func]
    pub fn get_current_hunger(&self) -> i32 {
        self.current_hunger
    }

    #[func]
    pub fn set_current_hunger(&mut self, hunger: i32) {
        self.current_hunger = hunger;
    }
}