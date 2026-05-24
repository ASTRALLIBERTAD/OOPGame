use godot::classes::{AnimatedSprite2D, CharacterBody2D, ICharacterBody2D,Input};
use godot::prelude::*;
use godot::global::Key;

use crate::rustplayer::Rustplayer;
use crate::entity::Entity;

#[derive(GodotClass)]
#[class(base=CharacterBody2D)]
struct Pet {
    base: Base<CharacterBody2D>,

    is_following: bool,
    speed: f32,
    stop_threshold: f32,
    follow_distance: f32,

    current_health: i32,
    max_health: i32,

    a_was_pressed: bool,
    d_was_pressed: bool,

    #[export]
    player: OnEditor<Gd<Rustplayer>>,

    #[export]
    sprite: OnEditor<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl ICharacterBody2D for Pet {

    fn init(base: Base<CharacterBody2D>) -> Self {

        Self {
            base,

            is_following: false,
            speed: 100.0,
            stop_threshold: 10.0,
            follow_distance: 100.0,

            current_health: 15,
            max_health: 15,
            
            a_was_pressed: false,
            d_was_pressed: false,
            
            player: OnEditor::default(),
            sprite: OnEditor::default(),
        }
    }

    fn physics_process(&mut self, _delta: f64) {

        let input = Input::singleton();

        let a_pressed = 

          input.is_physical_key_pressed(Key::A);

        let d_pressed =
          input.is_physical_key_pressed(Key::D);

        if a_pressed && !self.a_was_pressed {
    self.heal(1);
       }

       if d_pressed && !self.d_was_pressed {
          self.take_damage(1);
        }

        self.a_was_pressed = a_pressed;
        self.d_was_pressed = d_pressed;

        if !self.player.is_instance_valid() {
            return;
        }

        let player_position =
            self.player.get_global_position();

        let pet_position =
            self.base().get_global_position();

        let distance =
            pet_position.distance_to(player_position);

        if self.is_following {

            if distance > self.stop_threshold {

                self.sprite
                    .play_ex()
                    .name("run")
                    .done();

                self.move_toward_player();

                self.flip_sprite();
            }
            else {

                self.sprite
                    .play_ex()
                    .name("idle")
                    .done();

                self.is_following = false;

                self.stop_moving();
            }
        }
        else if distance > self.follow_distance {

            self.is_following = true;
        }
    }
}

#[godot_api]
impl Pet {
    #[allow(dead_code)]
    fn test_damage(&mut self) {

     self.take_damage(1);
    }
    #[allow(dead_code)]
    fn test_heal(&mut self) {

      self.heal(1);
    }


    fn update_health(&mut self, change: i32) {

        self.current_health =
            (
                self.current_health + change
            )
            .clamp(0, self.max_health);

        godot_print!(
            "Cat HP: {} / {}",
            self.current_health,
            self.max_health
        );

        if self.current_health <= 0 {

            godot_print!("Cat died");

            self.is_following = false;

            self.stop_moving();

            self.sprite
                .play_ex()
                .name("idle")
                .done();
        }
    }

    fn move_toward_player(&mut self) {

        let direction =
            (
                self.player.get_global_position()
                - self.base().get_global_position()
            )
            .normalized();

        let speed = self.speed;

         self.base_mut()
             .set_velocity(direction * speed);

        self.base_mut().move_and_slide();
    }

    fn stop_moving(&mut self) {

        self.base_mut()
            .set_velocity(Vector2::ZERO);

        self.base_mut().move_and_slide();
    }

    fn flip_sprite(&mut self) {

        if self.player.get_global_position().x
            < self.base().get_global_position().x
        {
            self.sprite.set_flip_h(true);
        }
        else {

            self.sprite.set_flip_h(false);
        }
    }
}

impl Entity for Pet {

    fn take_damage(&mut self, amount: i32) {

        self.update_health(-amount);
    }

    fn heal(&mut self, amount: i32) {

        self.update_health(amount);
    }

    fn is_alive(&self) -> bool {

        self.current_health > 0
    }
}