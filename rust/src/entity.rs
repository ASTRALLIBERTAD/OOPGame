use godot::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MobState {
    Idle,
    Aggro,
    Fleeing,
    Dead,
}   
#[allow(dead_code)]
pub trait Entity {
    // NOTE use i32 instead of f32, f32 is float or a decimal value
    fn take_damage(&mut self, amount: i32);
    fn heal(&mut self, amount: i32);
    fn is_alive(&self) -> bool;
}

pub trait HostileBehavior: Entity {
    fn aggro(&mut self, target: Vector2);
    fn chase(&mut self, target: Vector2, speed: f32);
    fn attack(&mut self, target: &mut dyn Entity);
}

pub trait NeutralBehavior: Entity {
    fn interact(&self) -> &'static str;
    fn become_hostile(&mut self);
}

pub trait PassiveBehavior: Entity {
    fn wander(&mut self);
    fn flee(&mut self, from: Vector2);
}
