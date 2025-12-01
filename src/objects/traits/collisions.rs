use bevy::{math::FloatPow, prelude::*};

pub enum Shapes {
    Circle(f32), //radius
                 //Square { width: f32 },
                 //Rectangle { width: f32, height: f32 },
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Position {
    pub x_prev: f32,
    pub y_prev: f32,
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position {
            x,
            y,
            x_prev: x,
            y_prev: y,
        }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        ((self.x - other.x).squared() + (self.y - other.y).squared()).sqrt()
    }

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

pub struct LineSegment<'a> {
    pos: &'a Position,
    pub a: f32,
    pub b: f32,
    pub c: f32,
} //(A,B,C)

impl<'a> LineSegment<'a> {
    fn distance_to_pt(&self, x: f32, y: f32) -> f32 {
        //https://www.splashlearn.com/math-vocabulary/distance-of-a-point-from-a-line#:~:text=The%20shortest%20distance%20between%20point%20and%20line,drawn%20from%20the%20point%20to%20the%20line.
        let factor = (self.a * x + self.b * y + self.c) / (self.a * self.a + self.b * self.b); //https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line
        let x_on_line = x - self.a * factor;
        let y_on_line = y - self.b * factor;

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

pub trait CollisionDetection {
    fn get_position(&self) -> Position;
    fn get_hitbox(&self) -> Shapes;

    //https://www.topcoder.com/thrive/articles/Geometry%20Concepts%20part%202:%20%20Line%20Intersection%20and%20its%20Applications
    fn minimum_distance(&self, two: &Position) -> Option<f32> {
        let one = self.get_position();
        let l1: LineSegment = one.gen_line_segment()?;
        let l2: LineSegment = two.gen_line_segment()?;

        let det = (l1.b * l2.a) - (l2.b * l1.a);
        if det != 0.0 {
            let int_x = (l2.b * l1.c - l1.b * l2.c) / det;
            let int_y = (l1.a * l2.c - l2.a * l1.c) / det; //these are intesections of the infinite lines

            if one.x.min(one.x_prev) >= int_x
                && one.x.max(one.x_prev) <= int_x
                && one.y.min(one.y_prev) >= int_y
                && one.y.max(one.y_prev) <= int_y
            {
                //then intersect
                return Some(0.0f32);
            }
        }

        //lines are parallel and so a1=a2 and b1=b2
        //or line segemtns do not intesect.
        //either way, solve by finding closest endpoint to the other line
        Some(
            l1.distance_to_pt(two.x, two.y)
                .min(l1.distance_to_pt(two.x_prev, two.y_prev)),
        )
    }

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
