use godot::classes::{HBoxContainer, IHBoxContainer, Texture2D};
use godot::prelude::*;

use crate::heart_display::HeartDisplay;

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct Heart {
    base: Base<HBoxContainer>,

    #[export]
    full_heart: OnEditor<Gd<Texture2D>>,

    #[export]
    half_heart: OnEditor<Gd<Texture2D>>,

    #[export]
    empty_heart: OnEditor<Gd<Texture2D>>,

    heart_list: Vec<Gd<HeartDisplay>>,

    #[export(range = (0.0, 20.0))]
    #[var(get = get_current_health, set = set_current_health)]
    current_health: i32,
}

#[godot_api]
impl IHBoxContainer for Heart {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,
            full_heart: OnEditor::default(),
            half_heart: OnEditor::default(),
            empty_heart: OnEditor::default(),
            heart_list: Vec::new(),
            current_health: i32::default(),
        }
    }

    fn ready(&mut self) {
        let heart_parents = self.base_mut().get_children();
        for i in heart_parents.iter_shared() {
            if let Ok(texture_rect) = i.try_cast::<HeartDisplay>() {
                self.heart_list.push(texture_rect);
                // Handle the TextureRect here
            }
        }
        godot_print!("Heart list length: {:?}", self.heart_list);
        godot_print!("Heart UI is ready!");
    }
}

#[godot_api]
impl Heart {
    pub fn set_heart_display(&mut self, change: i32) {
        godot_print!("updating health");

        let full_heart: &Gd<Texture2D> = self.full_heart.to_godot();
        let half_heart: &Gd<Texture2D> = self.half_heart.to_godot();
        let empty_heart: &Gd<Texture2D> = self.empty_heart.to_godot();

        //   Apply half-heart system (1 = half heart)
        self.current_health = (self.current_health + change).clamp(0, 20);

        godot_print!("current health (HP): {:?}", self.current_health);

        //  Convert HP → heart visuals
        let mut remaining_hp = self.current_health;

        for heart in &mut self.heart_list {
            let mut heart_node = heart.bind_mut();

            if remaining_hp >= 2 {
                //  full heart
                heart_node.set_health(2);
                heart_node.base_mut().set_texture(&full_heart.clone());
                remaining_hp -= 2;
            } else if remaining_hp == 1 {
                //  half heart
                heart_node.set_health(1);
                heart_node.base_mut().set_texture(&half_heart.clone());
                remaining_hp -= 1;
            } else {
                //  empty
                heart_node.set_health(0);
                heart_node.base_mut().set_texture(&empty_heart.clone());
            }
        }
    }

    #[func]
    pub fn get_current_health(&self) -> i32 {
        self.current_health
    }

    #[func]
    pub fn set_current_health(&mut self, health: i32) {
        self.current_health = health;
    }
}
