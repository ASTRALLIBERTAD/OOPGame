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
    pub current_health: i32,
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
    fn update_health(&mut self, change: i32) {
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

    pub fn damage(&mut self, damage: i32) {
        let mut remaining_damage = damage;

        for heart in self.heart_list.iter_mut().rev() {
            let mut heart_ref = heart.bind_mut();
            let mut current_heart_health = heart_ref.get_health();

            if current_heart_health == 2 {
                let damage_to_apply = remaining_damage.min(2);
                current_heart_health -= damage_to_apply;
                heart_ref.set_health(current_heart_health);
                remaining_damage -= damage_to_apply;
                godot_print!(
                    "damage is {:?} and current health {:?}",
                    damage,
                    self.current_health
                );
            } else if current_heart_health == 1 {
                let damage_to_apply = remaining_damage.min(1);
                current_heart_health -= damage_to_apply;
                heart_ref.set_health(current_heart_health);
                remaining_damage -= damage_to_apply;
                godot_print!(
                    "damage is {:?} and current health {:?}",
                    damage,
                    self.current_health
                );
            }

            if remaining_damage <= 0 {
                break;
            }
        }
        self.update_health(-damage);
    }

    pub fn heal(&mut self, heal: i32) {
        let mut remaining_heal = heal;
        let heart_parents = self.base_mut().get_children();
        for heart_display in heart_parents.iter_shared() {
            if let Ok(mut texture_rect) = heart_display.try_cast::<HeartDisplay>() {
                let mut current_heart_health = texture_rect.bind_mut().get_health();

                if current_heart_health == 0 {
                    let heal_to_apply = remaining_heal.min(2);
                    current_heart_health += heal_to_apply;
                    texture_rect.bind_mut().set_health(current_heart_health);
                    remaining_heal -= heal_to_apply;
                } else if current_heart_health == 1 {
                    let heal_to_apply = remaining_heal.min(1);
                    current_heart_health += heal_to_apply;
                    texture_rect.bind_mut().set_health(current_heart_health);
                    remaining_heal -= heal_to_apply;
                }

                godot_print!("heal is {:?}", heal);
            }

            if remaining_heal <= 0 {
                break;
            }
        }

        self.update_health(heal);
    }
}

