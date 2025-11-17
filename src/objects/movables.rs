use crate::objects::traits::collisions::{CollisionDetection, LineSegment, Position, Shapes};
use bevy::math::FloatPow;
use bevy::prelude::*;
use rand_distr::Normal;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::BTreeSet;
use std::default::Default;
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

static BLACKHOLECOUNT: AtomicU32 = AtomicU32::new(0);
static WORLDCOUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectType {
    BlackHole,
    World,
    Null,
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

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
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
pub struct CollisionFrame<'a> {
    array: Vec<CollisionSet<'a>>,
}

impl<'a> CollisionFrame<'a> {
    pub fn new() -> Self {
        CollisionFrame {
            array: Vec::<CollisionSet<'a>>::new(),
        }
    }

    //when appending, merges intersecting CollisionSets because if
    //0 collides with 2 and 1 collides with 2, then 0-1-2 all collide together
    //returns indicator: true means the push-ed CollisionSet was merged,
    //false means that it was a new unique collision
    pub fn push(&mut self, new: CollisionSet<'a>) -> bool {
        let mut found = false;

        for item in &mut self.array {
            match CollisionSet::merge_intersection(item, &new) {
                Some(n) => {
                    println!("{:?}", n);
                    *item = n;
                    found = true;
                    break;
                }
                None => {}
            }
        }

        if !found {
            self.array.push(new);
        }

        found
    }

    pub fn collect(&self) -> Option<Vec<Movable>> {
        let mut ret = Vec::<Movable>::new();

        if self.array.len() == 0 {
            return None;
        }

        //all CollisionSets in self.array are now guaranteed to be unique collisions
        for item in &self.array {
            match item.collide() {
                Some(n) => ret.push(n),
                None => {}
            }
        }

        Some(ret)
    }
}

#[derive(Component, Debug)]
pub struct CollisionSet<'a> {
    data: BTreeSet<&'a Movable>,
}

impl<'a> CollisionSet<'a> {
    pub fn new() -> Self {
        CollisionSet {
            data: BTreeSet::<&'a Movable>::new(),
        }
    }

    pub fn from_tuple(&mut self, tup: (&'a Movable, &'a Movable)) -> Self {
        let mut new = Self::new();
        new.append(tup.0);
        new.append(tup.1);

        new
    }

    pub fn append(&mut self, other: &'a Movable) -> bool {
        self.data.insert(other)
    }

    pub fn intersect(&self, other: &CollisionSet<'a>) -> Self {
        let set = self.data.intersection(&other.data);
        //let ret = CollisionSet::new();
        let mut new = BTreeSet::<&'a Movable>::new();
        for item in set {
            new.insert(item);
        }

        CollisionSet { data: new }
    }

    pub fn union(&self, other: &CollisionSet<'a>) -> Self {
        let union = self.data.union(&other.data);
        //let ret = CollisionSet::new();
        let mut new = BTreeSet::<&'a Movable>::new();
        for item in union {
            new.insert(item);
        }

        CollisionSet { data: new }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn merge_intersection(
        one: &CollisionSet<'a>,
        two: &CollisionSet<'a>,
    ) -> Option<CollisionSet<'a>> {
        let intersect = one.intersect(two);
        if !intersect.is_empty() {
            Option::Some(one.union(two))
        } else {
            Option::None
        }
    }

    pub fn collide(&self) -> Option<Movable> {
        let count = self.len();
        if count > 1 {
            //gotta have 2 obj to collide

            let mut cur = Movable::new_nulltype();

            for item in &self.data {
                println!("{:?}", item);
                cur = cur.handle_collision(item);
            }

            return Some(cur);
        }

        None
    }
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
    const MINIMUM_RADIUS: f32 = 1.0f32;
    const G: f32 = 1000_000_000.0;
    const EPSILON: f32 = 1000.0; //to pad on radius to prevent divide by zero possibilities
    const MAXACCELERATION: f32 = 1.0E4;
    const MAXVELOCITY: f32 = 10_000.0; //that would mean travel the length of the universe in 1 second

    pub fn new(otype: &ObjectType) -> Self {
        let id = match (otype) {
            ObjectType::BlackHole => BLACKHOLECOUNT.fetch_add(1, SeqCst),
            ObjectType::World => WORLDCOUNT.fetch_add(1, SeqCst),
            _ => 0,
        };

        Movable {
            id: ID(id),
            otype: *otype,
            ..default()
        }
    }

    pub fn new_nulltype() -> Self {
        Movable {
            id: ID(0),
            otype: ObjectType::Null,
            ..default()
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position.x = x;
        self.position.y = y;
        self
    }

    pub fn set_velocity(&mut self, vx: f32, vy: f32) -> &mut Self {
        if vx < 0.0 {
            self.velocity.vx = vx.max(-Movable::MAXVELOCITY);
        } else {
            self.velocity.vx = vx.min(Movable::MAXVELOCITY);
        }

        if vx < 0.0 {
            self.velocity.vy = vy.max(-Movable::MAXVELOCITY);
        } else {
            self.velocity.vy = vy.min(Movable::MAXVELOCITY);
        }

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
            ObjectType::BlackHole => {
                self.size.radius = (3.0f32 * mass).max(Movable::MINIMUM_RADIUS)
            } //https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
            ObjectType::World => {
                //https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1, Eq5
                if mass < 5.0 {
                    self.size.radius = (1.02f32 * mass.powf(0.27)).max(Movable::MINIMUM_RADIUS);
                } else if mass < 127.0 {
                    self.size.radius = (18.6f32 * mass.powf(-0.06)).max(Movable::MINIMUM_RADIUS);
                } else {
                    self.size.radius = (0.56 * mass.powf(0.67)).max(Movable::MINIMUM_RADIUS);
                }
            }
            _ => {}
        }
        println!("r={}", self.size.radius);
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
        /*println!(
            "{} {r} {a} {theta} {} {} {} {} {} {}",
            self.id.0,
            a * theta.cos(),
            a * theta.sin(),
            self.position.x,
            self.position.y,
            dy,
            dx
        );*/

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

        Movable::new(&ObjectType::BlackHole)
            .set_position(center_of_mass_x, center_of_mass_y)
            .set_velocity(new_velocity_x, new_velocity_y)
            .set_mass(new_mass)
            .build()
    }

    //self collided with all the members in others slice
    pub fn handle_collision(&self, two: &Self) -> Self {
        Movable::generate_blackhole(self, two)
    }

    //pub fn handle_collision(one: &Self, two: &Self) -> Self {
    //    Movable::generate_blackhole(one, two)
    //}
}

impl PartialEq for Movable {
    fn eq(&self, other: &Self) -> bool {
        //two objects cannot occupy the same exact location
        //(self.position.x == other.position.x) && (self.position.y == other.position.y)
        self.id.0 == other.id.0
    }
}

impl Eq for Movable {}
impl PartialOrd for Movable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Movable {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_index = self.get_id();
        let other_index = other.get_id();

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

impl Default for Movable {
    fn default() -> Self {
        Movable {
            id: ID(0),
            otype: ObjectType::BlackHole,
            position: Position {
                x: 0.0,
                y: 0.0,
                x_prev: 0.0,
                y_prev: 0.0,
            },
            velocity: Velocity { vx: 0.0, vy: 0.0 },
            size: Size {
                radius: 0.0,
                mass: 0.0,
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
