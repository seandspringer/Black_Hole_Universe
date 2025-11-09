use bevy::prelude::*;

pub enum Shapes {
    Circle(f32), //radius
                 //Square { width: f32 },
                 //Rectangle { width: f32, height: f32 },
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub trait CollisionDetection {
    fn get_position(&self) -> Position;
    fn get_hitbox(&self) -> Shapes;
    fn distance_to(&self, point: &Position) -> f32;

    fn collided(&self, other: &dyn CollisionDetection) -> bool {
        let my_hitbox = self.get_hitbox();

        let other_position = other.get_position();
        let other_hitbox = other.get_hitbox();

        let distance = self.distance_to(&other_position);

        match my_hitbox {
            Shapes::Circle(r1) => match other_hitbox {
                Shapes::Circle(r2) => distance <= r1 + r2,
            },
        }
    }
}
