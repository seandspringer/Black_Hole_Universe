use crate::objects::traits::collisions::{CollisionDetection, LineSegment, Position, Shapes};
use bevy::math::FloatPow;
use bevy::prelude::*;
use rand_distr::Normal;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::default::Default;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectType {
    BlackHole,
    World,
}

#[derive(Component, Debug)]
pub struct Acceleration {
    pub ax: f32, //% of speed of light
    pub ay: f32,
}

#[derive(Component, Debug)]
pub struct Velocity {
    pub vx: f32, //% of speed of light
    pub vy: f32,
}

#[derive(Component, Debug)]
pub struct Size {
    pub radius: f32, //radius = 3 * mass https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
    pub mass: f32,   //solar masses = 1.989x10^30 Kg
}

#[derive(Component, Debug)]
pub struct ID(u32);

#[derive(Component, Debug)]
pub struct Movable {
    id: ID,
    pub otype: ObjectType,
    pub position: Position,
    pub velocity: Velocity,
    pub size: Size,
}

#[derive(Component, Debug)]
pub struct MovableTuple<'a, 'b>(pub &'a Movable, pub &'b Movable);
impl<'a, 'b> MovableTuple<'a, 'b> {
    pub fn new(one: &'a Movable, two: &'b Movable) -> Self {
        MovableTuple(one, two)
    }
}
impl<'a, 'b> PartialEq for MovableTuple<'a, 'b> {
    fn eq(&self, other: &Self) -> bool {
        let ret = ((self.0 == other.0) || (self.0 == other.1))
            && ((self.1 == other.0) || (self.1 == other.1));
        ret
    }
}
impl<'a, 'b> Eq for MovableTuple<'a, 'b> {}
impl<'a, 'b> PartialOrd for MovableTuple<'a, 'b> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a, 'b> Ord for MovableTuple<'a, 'b> {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_index = std::cmp::min(self.0.get_id(), self.1.get_id());
        let other_index = std::cmp::min(other.0.get_id(), other.1.get_id());

        let ret = if my_index < other_index {
            Ordering::Less
        } else if my_index > other_index {
            Ordering::Greater
        } else {
            Ordering::Equal
        };

        ret
    }
}

impl Movable {
    const MINIMUM_RADIUS: f32 = 5.0f32;
    const G: f32 = 1000_000_000.0;
    const EPSILON: f32 = 1000.0; //to pad on radius to prevent divide by zero possibilities
    const MAXACCELERATION: f32 = 1.0E4;
    const MAXVELOCITY: f32 = 10_000.0; //that would mean travel the length of the universe in 1 second

    pub fn new(id: u32, otype: &ObjectType) -> Self {
        Movable {
            id: ID(id),
            otype: *otype,
            ..default()
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position.x = x;
        self.position.y = y;
        self
    }

    pub fn set_velocity(&mut self, vx: f32, vy: f32) -> &mut Self {
        self.velocity.vx = vx.min(Movable::MAXVELOCITY);
        self.velocity.vy = vy.min(Movable::MAXVELOCITY);
        self
    }

    pub fn set_size(&mut self, mass: f32, radius: f32) -> &mut Self {
        self.size.mass = mass;
        self.size.radius = radius;
        self
    }

    pub fn set_mass(&mut self, mass: f32) -> &mut Self {
        self.size.mass = mass;

        match self.otype {
            ObjectType::BlackHole => self.size.radius = 3.0f32 * mass, //https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
            ObjectType::World => {
                //https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1, Eq5
                if mass < 5.0 {
                    self.size.radius = 1.02f32 * mass.powf(0.27);
                } else if mass < 127.0 {
                    self.size.radius = 18.6f32 * mass.powf(-0.06);
                } else {
                    self.size.radius = 0.56 * mass.powf(0.67);
                }
            }
        }

        self
    }

    //for method build chaining
    pub fn build(&self) -> Movable {
        Movable {
            id: ID(self.id.0),
            otype: self.otype,
            position: Position {
                x: self.position.x,
                x_prev: self.position.x,
                y: self.position.y,
                y_prev: self.position.y,
            },
            velocity: Velocity {
                vx: self.velocity.vx,
                vy: self.velocity.vy,
            },
            size: Size {
                radius: self.size.radius,
                mass: self.size.mass,
            },
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id.0
    }

    pub fn calculate_acceleration(&self, other: &Self) -> Acceleration {
        let dx = other.position.x - self.position.x;
        let dy = other.position.y - self.position.y;
        let r = dx.squared() + dy.squared();
        let a =
            (Movable::G * other.size.mass / (r + Movable::EPSILON)).min(Movable::MAXACCELERATION);
        let theta = dy.atan2(dx);
        println!(
            "{} {r} {a} {theta} {} {} {} {} {} {}",
            self.id.0,
            a * theta.cos(),
            a * theta.sin(),
            self.position.x,
            self.position.y,
            dy,
            dx
        );

        Acceleration {
            ax: a * theta.cos(),
            ay: a * theta.sin(),
        }
    }

    pub fn update_location(&mut self, time_delta: f32) {
        self.position.x += self.velocity.vx * time_delta;
        self.position.y += self.velocity.vy * time_delta;
    }

    pub fn update_velocity(&self, others: &[&Movable], time: f32) -> Velocity {
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };

        for other in others {
            if self != *other {
                let cur = self.calculate_acceleration(other);
                acc.ax += cur.ax;
                acc.ay += cur.ay;
            }
        }

        Velocity {
            vx: self.velocity.vx + acc.ax * time,
            vy: self.velocity.vy + acc.ay * time,
        }
    }

    //private!
    fn generate_blackhole(one: &Self, two: &Self) -> Self {
        let new_mass = one.size.mass + two.size.mass;

        //use 2 body center of mass equation
        let center_of_mass_x = (one.size.mass * one.position.x + two.size.mass * two.position.x)
            / (one.size.mass + two.size.mass);
        let center_of_mass_y = (one.size.mass * one.position.y + two.size.mass * two.position.y)
            / (one.size.mass + two.size.mass);

        //add momentum becors then divide by new mass
        let new_velocity_x =
            ((one.size.mass * one.velocity.vx) + (two.size.mass * two.velocity.vx)) / new_mass;
        let new_velocity_y =
            ((one.size.mass * one.velocity.vy) + (two.size.mass * two.velocity.vy)) / new_mass;

        Movable::new(one.id.0, &ObjectType::BlackHole)
            .set_position(center_of_mass_x, center_of_mass_y)
            .set_velocity(new_velocity_x, new_velocity_y)
            .set_mass(new_mass)
            .build()
    }

    //private!
    //fn generate_world(one: Self) -> Option<Self> {
    //    // each world splits in half

    //}

    pub fn handle_collision(one: &Self, two: &Self) -> Self {
        //let myType = one.otype;
        //let otherType = two.otype;

        //if myType == ObjectType::BlackHole || otherType == ObjectType::BlackHole {
        //return blackhole
        Movable::generate_blackhole(one, two)
        //} else {
        //world on world = 2 worlds
        //}
    }
}

impl PartialEq for Movable {
    fn eq(&self, other: &Self) -> bool {
        //two objects cannot occupy the same exact location
        //(self.position.x == other.position.x) && (self.position.y == other.position.y)
        (self.id.0 == other.id.0) && (self.otype == other.otype)
    }
}

impl Default for Movable {
    fn default() -> Self {
        Movable {
            id: ID(0),
            otype: ObjectType::BlackHole,
            position: Position {
                x: 0.0,
                y: -5000.0,
                x_prev: 0.0,
                y_prev: -5000.0,
            },
            velocity: Velocity {
                vx: 0.0,
                vy: 1000.0,
            },
            size: Size {
                radius: 50.0,
                mass: 20.0,
            },
        }
    }
}

impl CollisionDetection for Movable {
    fn get_position(&self) -> Position {
        self.position
    }

    fn get_hitbox(&self) -> Shapes {
        Shapes::Circle(self.size.radius)
    }

    //fn distance_to(&self, point: &Position) -> f32 {
    //    ((self.position.x - point.x) * (self.position.x - point.x)
    //        + (self.position.y - point.y) * (self.position.y - point.y))
    //        .sqrt()
    //}
}
