use rand::prelude::*;
use rand_distr::{Distribution, Normal};

#[allow(dead_code)]
pub enum GaussBoundary {
    None,
    Lower(f32),            //clamp
    Upper(f32),            //clamp
    ClampBoth((f32, f32)), //lower, upper
    WrapBoth((f32, f32)),  //lower, upper
}

pub struct Gauss {
    generator: ThreadRng,
    distribution: Normal<f32>,
    boundary: GaussBoundary,
}

#[allow(dead_code)]
impl Gauss {
    pub fn new(mean: f32, std: f32, boundary: GaussBoundary) -> Gauss {
        assert!(std > 0.0);

        Gauss {
            generator: rand::rng(),
            distribution: Normal::new(mean, std).unwrap(),
            boundary,
        }
    }

    pub fn update(&mut self, mean: f32, std: f32, boundary: GaussBoundary) {
        assert!(std > 0.0);

        self.distribution = Normal::new(mean, std).unwrap();
        self.boundary = boundary;
    }

    pub fn sample(&mut self) -> f32 {
        let value = self.distribution.sample(&mut self.generator);
        match self.boundary {
            GaussBoundary::None => value,
            GaussBoundary::Lower(n) => value.max(n),
            GaussBoundary::Upper(n) => value.min(n),
            GaussBoundary::ClampBoth((lower, upper)) => value.max(lower).min(upper),
            GaussBoundary::WrapBoth((lower, upper)) => {
                let mut wrapped = value;

                if value < lower {
                    wrapped = value - lower + upper;
                }
                if value > upper {
                    wrapped = value - upper + lower;
                }

                wrapped
            }
        }
    }
}
