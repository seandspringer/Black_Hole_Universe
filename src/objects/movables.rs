//! Movables.rs
//!
//! Movables.rs is the workhorse of this crate. It contains logic for any
//! moving object in the universe and defines that movement.
//!
//! In short, here are a few topics addressed in movables.rs:
//! 1. Definitions tied to an objects ability to move
//! 2. Physics calculations for gravity and acceleration
//! 3. Collision logic and helper data structures

use crate::objects::gamestate::UNIVERSE_SIZE;
use crate::objects::traits::collisions::{CollisionDetection, Position, Shapes};
use bevy::math::FloatPow;
use bevy::prelude::*;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::BTreeSet;
use std::default::Default;
use std::f32::consts::FRAC_PI_4;
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

/// these atomics are used to track the number of spawned objects
/// and assign unique IDs for lookup evaluations to each new spawned object
static BLACKHOLECOUNT: AtomicU32 = AtomicU32::new(0);
static WORLDCOUNT: AtomicU32 = AtomicU32::new(0);

/// ObjectType enum: Component
///
/// used within the Movable struct to define the type of object represented
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectType {
    BlackHole,
    World,
}

/// Acceleration struct: Component
///
/// used within the Movable struct to define the object's
/// current x and y acceleration components
#[derive(Component, Debug)]
pub struct Acceleration {
    pub ax: f32, //% of speed of light
    pub ay: f32,
}

/// Velocity struct: Component
///
/// used within the Movable struct to define the object's
/// current x and y velocity components
#[derive(Component, Debug)]
pub struct Velocity {
    pub vx: f32, //% of speed of light
    pub vy: f32,
}

/// Size struct: Component
///
/// used within the Movable struct to define the object's
/// current size and mass. Note, these properties can be connected
/// but are not explicity required to be so
#[derive(Component, Debug)]
pub struct Size {
    pub radius: f32, //radius = 3 * mass https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
    pub mass: f32,   //solar masses = 1.989x10^30 Kg
}

/// ID struct: Component
///
/// used within the Movable struct to define the object's
/// unique identifier (to prevent self-evaluation, for example)
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ID(u32);

/// Movable struct: Component
///
/// this is the main struct of the simulation and is passed
/// around and manipulated in many different places. You can expect
/// that this structs is updated for every object in the universe at
/// every frame
#[derive(Component, Debug)]
pub struct Movable {
    id: ID,
    pub otype: ObjectType,
    pub position: Position,
    pub velocity: Velocity,
    pub size: Size,
}

/// CollisionResult enum
///
/// Collisions are hard! This enum is returned from a CollisionFrame
/// evaluation and contains the result of the collision as either a
/// - None: no objects returned
/// - Single(Movable): collision resulted in a single resultant object = Movable
/// - NSize(Vec<Movable>): collision resulted in 2+ resultant objects = Vec<Movable>
pub enum CollisionResult {
    None,
    Single(Movable),
    NSize(Vec<Movable>),
}

/// CollisionSet struct: Component
///
/// A CollisionSet wraps a BTreeSet of Movable references and represents
/// a set of 2+ unique Movable objects which have been calculated to have
/// collided within the last frame. Most of this abstraction is built around
/// handling muliple / chain reaction collisions properly. Note the private
/// internal data structure: interactions are limited to impl'd methods
#[derive(Component, Debug)]
pub struct CollisionSet<'a> {
    data: BTreeSet<&'a Movable>,
}

impl<'a> CollisionSet<'a> {
    /// Constructor
    ///
    /// returns a new CollisionSet with an empty-initialized internal BTreeSet
    pub fn new() -> Self {
        CollisionSet {
            data: BTreeSet::<&'a Movable>::new(),
        }
    }

    /// fn append
    ///
    /// Append wraps the insert method of the BTreeSet and was named `append` bc
    /// it felt more approiate than insert in this context
    pub fn append(&mut self, other: &'a Movable) -> bool {
        self.data.insert(other)
    }

    /// fn intersect
    ///
    /// checks for the intersection of the set defined by self and other,
    /// returning a new CollisionSet with the intersection results
    pub fn intersect(&self, other: &CollisionSet<'a>) -> Self {
        let set = self.data.intersection(&other.data);
        let mut new = BTreeSet::<&'a Movable>::new();

        for item in set {
            new.insert(item);
        }

        CollisionSet { data: new }
    }

    /// fn union
    ///
    /// checks for the union of the set defined by self and other,
    /// returning a new CollisionSet with the union results
    pub fn union(&self, other: &CollisionSet<'a>) -> Self {
        let union = self.data.union(&other.data);
        let mut new = BTreeSet::<&'a Movable>::new();

        for item in union {
            new.insert(item);
        }

        CollisionSet { data: new }
    }

    /// fn len
    ///
    /// returns the number of Movable references in this set
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// fn is_empty
    ///
    /// returns true if the number of Movable references in this
    /// set is 0, false otherwise
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// fn merge_intersection
    ///
    /// this method was crucial to the proper collision function. Imagine,
    /// after a frame, we have determined that 2 collisions have occured. Labeling
    /// the Movable objects with capital letters, a valid descriptor of the 2 collisions
    /// could be: {`A` collides with `B`} and {`B` collides with `C`}: this is a common "chain reaction"
    /// style of collision. this method checks for common Movables with different collisions (using the
    /// intersect methods above) and if found, combines them into one collision (using the union methods above).
    /// In our example, this would convert the 2 collisions into 1: {`A` collides with `B` and `C`}
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

    /// fn collide
    ///
    /// fn collide physically produces the action of the collision of all the Movables
    /// within this set. This is performed by collecting the Movables into a vector and
    /// calling the static Movable::process_collisions method on the collection
    pub fn collide(&self) -> CollisionResult {
        let count = self.len();

        //gotta have 2 obj to collide
        if count > 1 {
            let mut v = Vec::<&&'a Movable>::new();

            for item in &self.data {
                v.push(item);
            }

            return Movable::process_collisions(&v);
        }

        CollisionResult::None
    }
}

/// CollisionFrame struct: Component
///
/// A CollisionFrame is a collection of CollisionSets (defined above) representing
/// all collisions determined during the current frame. Note, the internal data structure
/// is private: all interaction with this object should be performed via its methods
#[derive(Component, Debug)]
pub struct CollisionFrame<'a> {
    array: Vec<CollisionSet<'a>>,
}

impl<'a> CollisionFrame<'a> {
    /// Constructor
    ///
    /// returns a new CollisionFrame with an empty internal data structure
    pub fn new() -> Self {
        CollisionFrame {
            array: Vec::<CollisionSet<'a>>::new(),
        }
    }

    /// fn push
    ///
    /// when appending, this function merges intersecting CollisionSets because if
    /// 0 collides with 2 and 1 collides with 2, then 0-1-2 all collide together
    /// returns indicator: true means the push-ed CollisionSet was merged,
    /// false means that it was a new unique collision
    pub fn push(&mut self, new: CollisionSet<'a>) -> bool {
        let mut found = false;

        for item in &mut self.array {
            if let Some(n) = CollisionSet::merge_intersection(item, &new) {
                *item = n;
                found = true;
                break;
            }
        }

        if !found {
            self.array.push(new);
        }

        found
    }

    /// fn collect
    ///
    /// performs the collisions for all the objects of the frame. Returns either
    /// 1. CollisionResult::None => no collisions during this frame
    /// 2. CollisionResult::Single(n) => all collisions resulted in a single resultant object, n
    /// 3. CollisionResult::NSize(Vec!) => collisions resulted in 2+ resultant objects
    pub fn collect(&self) -> CollisionResult {
        let mut ret = Vec::<Movable>::new(); //flatten

        if self.array.is_empty() {
            return CollisionResult::None;
        }

        //all CollisionSets in self.array are now guaranteed to be unique collisions
        for item in &self.array {
            match item.collide() {
                CollisionResult::Single(n) => ret.push(n),
                CollisionResult::NSize(mut n) => ret.append(&mut n),
                CollisionResult::None => {}
            }
        }

        CollisionResult::NSize(ret)
    }
}

/// impl block for the Movable struct. Most of the physics and actual motion occurs
/// within these methods
impl Movable {
    /// Constant vars used for boundaries or calculations
    const MINIMUM_RADIUS: f32 = 1.0f32;
    const G: f32 = 100_000_000.0;
    const EPSILON: f32 = 1000.0; //to pad on radius to prevent divide by zero possibilities
    const MAXACCELERATION: f32 = 1.0E4;
    const MAXVELOCITY: f32 = 10_000.0; //that would mean travel the length of the universe in 1 second

    /// Constructor
    ///
    /// given an input ObjectType, returns a new Movable object. The default Trait is used to specify
    /// some of the detailed parameters. Note that each time this method returns, a new object ID is
    /// generated.
    ///
    /// This method is intended to be chained with the following intialization methods
    pub fn new(otype: &ObjectType) -> Self {
        let id = match otype {
            ObjectType::BlackHole => BLACKHOLECOUNT.fetch_add(1, SeqCst),
            ObjectType::World => WORLDCOUNT.fetch_add(1, SeqCst),
        };

        Movable {
            id: ID(id),
            otype: *otype,
            ..default()
        }
    }

    /// fn set_position: chain
    ///
    /// updates the objects x and y coordinates in the universe
    /// This method is intended to be chained with the following intialization methods
    pub fn set_position(&mut self, x: f32, y: f32) -> &mut Self {
        self.position.x = x;
        self.position.y = y;
        self
    }

    /// fn set_velocity: chain
    ///
    /// updates the objects x and y velocity in the universe, bounded by Movable::MAXVELOCITY constant
    /// This method is intended to be chained with the following intialization methods
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

    /// fn set_size: chain
    ///
    /// updates the objects mass and radius, unbounded
    /// This method is intended to be chained with the following intialization methods
    pub fn set_size(&mut self, mass: f32, radius: f32) -> &mut Self {
        self.size.mass = mass;
        self.size.radius = radius;
        self
    }

    /// fn set_mass: chain
    ///
    /// updates the objects mass and radius by calculating the radius via the supplied mass.
    /// See links in body below for reference to the coefficients used.
    /// This method is intended to be chained with the following intialization methods
    pub fn set_mass(&mut self, mass: f32) -> &mut Self {
        self.size.mass = mass;

        match self.otype {
            ObjectType::BlackHole => {
                self.size.radius = (3.0f32 * mass).max(Movable::MINIMUM_RADIUS)
            } //https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
            ObjectType::World => {
                //https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1, Eq5
                if mass <= 5.0 {
                    self.size.radius = (1.02f32 * mass.powf(0.27)).max(Movable::MINIMUM_RADIUS);
                } else if mass <= 127.0 {
                    self.size.radius = (18.6f32 * mass.powf(-0.06)).max(Movable::MINIMUM_RADIUS);
                } else {
                    self.size.radius = (0.56 * mass.powf(0.67)).max(Movable::MINIMUM_RADIUS);
                }
            }
        }

        self
    }

    /// fn set_radius: chain
    ///
    /// inverse function of above: updates the objects radius and mass by calculating the mass from the supplied radius.
    /// This method is intended to be chained with the following intialization methods
    pub fn set_radius(&mut self, radius: f32) -> &mut Self {
        self.size.radius = radius;

        match self.otype {
            ObjectType::BlackHole => {
                self.size.mass = radius / 3.0;
            } //https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
            ObjectType::World => {
                //https://www.aanda.org/articles/aa/full_html/2024/06/aa48690-23/aa48690-23.html#F1, Eq5

                if radius < 1.575151 {
                    self.size.mass = (radius / 1.02).powf(100.0 / 27.0); //(1.02f32 * mass.powf(0.27)).max(Movable::MINIMUM_RADIUS);
                } else if radius < 13.90864 {
                    self.size.mass = (radius / 18.6).powf(-100.0 / 6.0); //(18.6f32 * mass.powf(-0.06)).max(Movable::MINIMUM_RADIUS);
                } else {
                    self.size.mass = (radius / 0.56).powf(100.0 / 67.0) //(0.56 * mass.powf(0.67)).max(Movable::MINIMUM_RADIUS);
                }
            }
        }

        self
    }

    /// fn build: chain
    ///
    /// ends the chaining process by returning a new object containing the
    /// results of the chaining methods defined above
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

    /// fn get_id
    ///
    /// exterior getter for the ID parameter
    pub fn get_id(&self) -> u32 {
        self.id.0
    }

    /// fn calculate_acceleration
    ///
    /// calculates the x and y component of acceleration induced on self by other
    /// using Netwon's equations of motion and gravity. Note that because this
    /// universe is spherical (it wraps around on itself), this function will
    /// choose the proper direction of acceleration by using the shortest distance between
    /// self and other: either the visual straight line, or the wrapped around line
    pub fn calculate_acceleration(&self, other: &Self) -> Acceleration {
        let dx_straight = other.position.x - self.position.x;
        let wrap_dx = UNIVERSE_SIZE - dx_straight.abs();

        let dy_straight = other.position.y - self.position.y;
        let wrap_dy = UNIVERSE_SIZE - dy_straight.abs();

        let mut dx = dx_straight;
        let mut dy = dy_straight;

        if wrap_dx < dx_straight.abs() {
            //want to invert sign
            if dx_straight < 0.0 {
                dx = wrap_dx;
            } else {
                dx = -wrap_dx;
            }
        }
        if wrap_dy < dy_straight.abs() {
            //want to invert sign
            if dy_straight < 0.0 {
                dy = wrap_dy;
            } else {
                dy = -wrap_dy;
            }
        }

        let r = dx.squared() + dy.squared();

        let a =
            (Movable::G * other.size.mass / (r + Movable::EPSILON)).min(Movable::MAXACCELERATION);
        let theta = dy.atan2(dx);

        Acceleration {
            ax: a * theta.cos(),
            ay: a * theta.sin(),
        }
    }

    /// fn update_location
    ///
    /// position is velocity * time and so this function updates
    /// self's position using self's velocity and the supplied time interval
    /// (likely the time between successive frames)
    pub fn update_location(&mut self, time_delta: f32) {
        self.position.x += self.velocity.vx * time_delta;
        self.position.y += self.velocity.vy * time_delta;
    }

    /// fn update_velocity
    ///
    /// given a slice of all other Movables in the universe, calculates the x and y components of
    /// acceleration on self due to the gravity of all the other objects. The accelerations are
    /// vector summed and then the supplied time interval is used to calculate the new velocity
    /// for the next frame: v = v + a * t
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

    /// fn generate_blackhole:  static, private!
    ///
    /// given 2 movables, returns a single Movable using the black hole mechanics
    fn generate_blackhole(one: &Self, two: &Self) -> Self {
        let new_mass = one.size.mass + two.size.mass;

        //use 2 body center of mass equation
        let center_of_mass_x = (one.size.mass * one.position.x + two.size.mass * two.position.x)
            / (one.size.mass + two.size.mass);
        let center_of_mass_y = (one.size.mass * one.position.y + two.size.mass * two.position.y)
            / (one.size.mass + two.size.mass);

        //add momentum because then divide by new mass
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

    /// fn split_planet:  todo()!, private!
    ///
    /// currently unused method intended to split a planet into two planets,
    /// redistributing mass and radius to the split pair. Reserved for future use
    fn split_planet(&self) -> (Self, Self) {
        let new_velocity = (self.velocity.vx.squared() + self.velocity.vy.squared()).sqrt() / 2.0;
        let theta = self.velocity.vy.atan2(self.velocity.vx);
        let new_radius = self.size.radius / 2.0;
        let angle_offset = FRAC_PI_4; //45 deg in rad 

        //need to move these new planets so that they are outside eachother's collision zone
        let new_theta = theta + angle_offset;
        let p1 = Movable::new(&ObjectType::BlackHole)
            .set_position(
                self.position.x + new_radius * new_theta.cos(),
                self.position.y + new_radius * new_theta.sin(),
            )
            .set_velocity(
                new_velocity * new_theta.cos(),
                new_velocity * new_theta.sin(),
            )
            .set_radius(new_radius)
            .build();

        let new_theta = theta - angle_offset;
        let p2 = Movable::new(&ObjectType::BlackHole)
            .set_position(
                self.position.x - new_radius * new_theta.cos(),
                self.position.y - new_radius * new_theta.sin(),
            )
            .set_velocity(
                new_velocity * new_theta.cos(),
                new_velocity * new_theta.sin(),
            )
            .set_radius(new_radius)
            .build();

        (p1, p2)
    }

    /// fn process_collisions: static
    ///
    /// given a slice of Movable references all involved in a collision together,
    /// process the mathematics of the collision and returns the resultant object(s)
    /// as a CollisionResults enum variant
    pub fn process_collisions(items: &[&&Movable]) -> CollisionResult {
        let count = items.len();
        if count == 0 {
            return CollisionResult::None;
        }

        let bh_count: i32 = items
            .iter()
            .map(|x| {
                if x.otype == ObjectType::BlackHole {
                    1
                } else {
                    0
                }
            })
            .sum();

        if bh_count > 0 {
            //then the result must be a bh
            let mut cur = Movable::generate_blackhole(items[0], items[1]);

            for item in items.iter().take(count).skip(2) {
                cur = Movable::generate_blackhole(&cur, item); //like a cumsum
            }

            CollisionResult::Single(cur)
        } else {
            //only planets in this collision
            let mut vec = Vec::<Movable>::new();
            for item in items.iter().take(count) {
                let (p1, p2) = item.split_planet();
                vec.push(p1);
                vec.push(p2);
            }

            CollisionResult::NSize(vec)
        }
    }
}

/// the PartialEq trait is implemented for Movable so that
/// Movable references can be used in BTreeSets
impl PartialEq for Movable {
    fn eq(&self, other: &Self) -> bool {
        self.id.0 == other.id.0
    }
}

/// the Eq trait is implemented for Movable so that
/// Movable references can be used in BTreeSets
impl Eq for Movable {}

/// the PartialOrd trait is implemented for Movable so that
/// Movable references can be used in BTreeSets
impl PartialOrd for Movable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// the Ord trait is implemented for Movable so that
/// Movable references can be used in BTreeSets
impl Ord for Movable {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_index = self.get_id();
        let other_index = other.get_id();

        if my_index < other_index {
            Ordering::Less
        } else if my_index > other_index {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

/// Default is implemented for Movable mustly for convienence
/// during the object building chain as described above
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

/// Collision is a trait defined within this crate (see collision.rs)
/// objects that can collide with other objects should impl this trait
/// and define at least these 2 methods
impl CollisionDetection for Movable {
    /// defines how to describe an objects current position
    fn get_position(&self) -> Position {
        self.position
    }

    /// describes how to define the objects hit box as a geometric shape
    fn get_hitbox(&self) -> Shapes {
        Shapes::Circle(self.size.radius)
    }
}
