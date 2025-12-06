//! Collisions.rs
//!
//! collisions module defines mechanics for determine whether a collision has occured
//! given two objects current frame position and previous frame position. Complexity
//! arises when two objects are moving at sufficient velocity compared to the frame rate
//! so that the pass through eachother completly within that frame. Line segment logic
//! is implemented herein to determine if two objects passed through eachother between frames

use bevy::{math::FloatPow, prelude::*};

/// Shapes enum: todo()!
///
/// implements a geometric shape used to determine a collision by defining the
/// boundary of the object via this geometry primitive. ToDo: implement more shapes
/// to the universe
pub enum Shapes {
    Circle(f32), //radius
                 //Square { width: f32 },
                 //Rectangle { width: f32, height: f32 },
}

/// Position struct: Component
///
/// The position of the object: Note this is used within the Movable struct.
/// the need to store the previous and current location will hopefully become
/// evident below
#[derive(Component, Copy, Clone, Debug)]
pub struct Position {
    pub x_prev: f32,
    pub y_prev: f32,
    pub x: f32,
    pub y: f32,
}

/// helper methods for calculating position related calculations.
///
/// the allow(dead_code) because there are todo()! methods within
#[allow(dead_code)]
impl Position {
    /// Constructor
    ///
    /// set's current and prev to same value, intentional
    pub fn new(x: f32, y: f32) -> Position {
        Position {
            x,
            y,
            x_prev: x,
            y_prev: y,
        }
    }

    /// fn distance_to
    ///
    /// calculate Euclidean distance between self and other
    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).squared() + (self.y - other.y).squared()).sqrt()
    }

    /// fn gen_lin_segment
    ///
    /// given self's current and previous position, generate a LineSegment
    /// from those endpoints. Returns an Option<LineSegment> which can only
    /// return None if the objects previous and current position are the same,
    /// indicating that it hasn't moved yet
    pub fn gen_line_segment(&'_ self) -> Option<LineSegment<'_>> {
        //https://www.sunshine2k.de/articles/algorithm/line2d/linerep2d.html
        let a: f32 = self.y_prev - self.y;
        let b = self.x - self.x_prev;
        let c = self.x * self.y_prev - self.x_prev * self.y;

        if a == 0.0 && b == 0.0 {
            None
        } else {
            Some(LineSegment { pos: self, a, b, c })
        }
    }
}

/// LineSegment struct
///
/// represents a finite line segment in Standard (Cartesian) form
/// a*x + b*y = c
/// Note: pos is a borrowed reference with lifetime 'a
pub struct LineSegment<'a> {
    pos: &'a Position,
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

/// impl block for LinSegment used to calculate distance via method interface
impl<'a> LineSegment<'a> {
    /// fn distance_to_pt
    ///
    /// returns the nearest distance of this line segment to the given point.
    /// This function is used to determine if an intersection occured between frames
    fn distance_to_pt(&self, x: f32, y: f32) -> f32 {
        //https://www.splashlearn.com/math-vocabulary/distance-of-a-point-from-a-line#:~:text=The%20shortest%20distance%20between%20point%20and%20line,drawn%20from%20the%20point%20to%20the%20line.
        let factor = (self.a * x + self.b * y + self.c) / (self.a * self.a + self.b * self.b); //https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line
        let x_on_line = x - self.a * factor; //point on line closest to the given point
        let y_on_line = y - self.b * factor;
        // the point (x_on_line,y_on_line) can either be one of the endpoints or somewhere
        // in the interior of the line segment:

        //now need to check that x,y is on this line SEGMENT (self.x, self.y), (self.x_prev, self.y_prev)
        let within_x: bool = self.pos.x.min(self.pos.x_prev) <= x_on_line
            && self.pos.x.max(self.pos.x_prev) >= x_on_line;
        let within_y: bool = self.pos.y.min(self.pos.y_prev) <= y_on_line
            && self.pos.y.max(self.pos.y_prev) >= y_on_line;

        if within_x & within_y {
            (self.a * x + self.b * y + self.c).abs() / (self.a * self.a + self.b * self.b).sqrt() //distance
        } else {
            //must be one endpoint is closest to this point
            let d1 = (self.pos.x - x).squared() + (self.pos.y - y).squared();
            let d2 = (self.pos.x_prev - x).squared() + (self.pos.y_prev - y).squared();
            if d1 < d2 { d1.sqrt() } else { d2.sqrt() }
        }
    }
}

/// CollisionDetection Trait
///
/// Implement this trait for any object that requires collision detection be
/// calculated. Implementers need only define the top two functions
/// 1. fn get_position(&self) -> Position - simply defines the position of the object. This can be center of mass, center of geometry, etc
/// 2. fn get_hitbox(&self) -> Shapes - this returns a primitive geometric shape enum with internal data describing the size of the shape.
///    together, Position and Shapes defines the necessary components to implement the collision detection calculations
pub trait CollisionDetection {
    /// fn get_position(&self) -> Position : Abstract!
    ///
    /// Must be defined by implementor. Typically recommend to return the geometric center of the
    /// implementing object
    fn get_position(&self) -> Position;

    /// fn get_hitbox(&self) -> Shapes : Abstract!
    ///
    /// Must be defined by the implementor. The Shape variant and internal data should define the
    /// hit box of the object (typically it's area in 2D or volume in 3D).
    fn get_hitbox(&self) -> Shapes;

    /// fn minimum_distance(&self, two: &Position) -> Option<f32>
    ///
    /// Given a Position, determines the minimum distance from the two line segments
    /// defined by the Positions (x,y) and (x_prev, y_prev) parameters. This is performed by
    /// 1. Calculating the determinent of the system of equations; if = 0 then the lines are parallel
    /// 2. If parallel, then the LineSegment::distance_to_pt method is invoked to find the nearest point
    ///    to the lines
    /// 3. If lines do intersect, then the intersection point is calculated and checked to be within the segment
    /// 3. If interseciton pt is outside the segments, the LineSegment::distance_to_pt method is again invoked
    ///
    /// See the following link for reference:
    /// https://www.topcoder.com/thrive/articles/Geometry%20Concepts%20part%202:%20%20Line%20Intersection%20and%20its%20Applications
    fn minimum_distance(&self, two: &Position) -> Option<f32> {
        let one = self.get_position();
        let l1: LineSegment = one.gen_line_segment()?;
        let l2: LineSegment = two.gen_line_segment()?;

        let det = (l1.b * l2.a) - (l2.b * l1.a);

        // if det == 0 then lines are parallel
        if det != 0.0 {
            // if not parallel, then they must intersect eventually...
            // these are intesections of the infinite lines:
            let int_x = (l2.b * l1.c - l1.b * l2.c) / det;
            let int_y = (l1.a * l2.c - l2.a * l1.c) / det;

            // now need to check if that intersection points is one the segment:
            if one.x.min(one.x_prev) >= int_x
                && one.x.max(one.x_prev) <= int_x
                && one.y.min(one.y_prev) >= int_y
                && one.y.max(one.y_prev) <= int_y
            {
                //then they intersect, so minimum distance is 0
                return Some(0.0f32);
            }
        }

        // lines are parallel and so a1=a2 and b1=b2 or line segements do not intesect.
        // either way, solve by finding closest endpoint to the other line (the closest
        // point must be one of the endpoints now)
        Some(
            l1.distance_to_pt(two.x, two.y)
                .min(l1.distance_to_pt(two.x_prev, two.y_prev)),
        )
    }

    /// fn collided(&self, other: &dyn CollisionDetection) -> bool
    ///
    /// given a trait object of this same trait, returns a boolean indicating
    /// whether the two CollisionDetection trait objects have collided. Calculations
    /// are performed using the methods within this trait and can be summarized as follows:
    /// 1. Get the Position and hitbox Shapes for self and other
    /// 2. Calculate the minimum distance between the two positions using the logic
    ///    described in detail above
    /// 3. If this minimum distance is within the intersection region of the hitboxes, returns true
    ///    and otherwise false
    fn collided(&self, other: &dyn CollisionDetection) -> bool {
        let my_hitbox = self.get_hitbox();

        let other_position = other.get_position();
        let other_hitbox = other.get_hitbox();

        let min_r = self.minimum_distance(&other_position);
        if min_r.is_none() {
            return false;
        }

        let min_r = min_r.unwrap();

        match my_hitbox {
            Shapes::Circle(r1) => match other_hitbox {
                Shapes::Circle(r2) => min_r <= r1 + r2,
            },
        }
    }
}
