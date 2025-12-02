//! Gauss.rs
//!
//! The gauss module defines the Gauss (Normal) propability distribution sampling
//! and allows for building the Gauss object which has some functionality for
//! - setting the mean and stdev of the Normal distribution
//! - randomly sample the built Normal distribution
//! - enfouce boundary conditions on the sampled value

use rand::prelude::*;
use rand_distr::{Distribution, Normal};

/// GaussBoundary Enum
///
/// Defines boundary conditions on Normal distribution sampled values.  
/// Contains the following variants:
/// 1. None - no boundary conditions enforced
/// 2. Lower(f32) - only enforce a clamped lower boundary of value f32
/// 3. Upper(f32) - only enforce a clamped upper boundary of value f32
/// 4. ClampBoth((f32,f32)) - enforce clamped double-ended boundary conditions of (lower, upper)
/// 5. WrapBoth((f32,f32)) - enforce wrapped double-ended boundary codntions of (lower,upper)  
/// Note, that a value which bases outside of (lower, upper) will wrap around to the other side
///
/// allow(dead_code) used to prevent warnings on unused Lower and Upper variants: intended in future use
#[allow(dead_code)]
pub enum GaussBoundary {
    None,
    Lower(f32),            //clamp
    Upper(f32),            //clamp
    ClampBoth((f32, f32)), //lower, upper
    WrapBoth((f32, f32)),  //lower, upper
}

/// Gauss Struct
///
/// Main object of this module and is used to setup and generate Normal distribution sampling
/// - generator = rand::rngs::thread::ThreadRng used to reference the local random number generator
/// - distrubtion = Normal<f32> gauss function of form: a*exp(- x^2 / (2*std^2))
/// - boundary = GaussBoundary defining limits and how to enforce the boundary conditions
///
/// Note: all members are private; use impl methods to interact
pub struct Gauss {
    generator: ThreadRng,
    distribution: Normal<f32>,
    boundary: GaussBoundary,
}

/// impl Gauss block
///
/// provides methods to produce a new Normal distrubtion and to sample the  
/// distribution for a new random sample
impl Gauss {
    /// fn new returns a Guass struct.
    ///
    /// - mean dictates the center of the Normal distrbution
    /// - std dicates the standard deviation (width) of the Normal distribution
    /// - boundary defines an allowed range and how to handle values sampled outside of said range
    pub fn new(mean: f32, std: f32, boundary: GaussBoundary) -> Gauss {
        assert!(std > 0.0);

        Gauss {
            generator: rand::rng(),
            distribution: Normal::new(mean, std).unwrap(),
            boundary,
        }
    }

    /// fn sample returns an f32 which is the Normal distrubtion sampled random number, with boundary  
    /// conditions enforced, if applicable.
    pub fn sample(&mut self) -> f32 {
        let value = self.distribution.sample(&mut self.generator);
        match self.boundary {
            GaussBoundary::None => value,
            GaussBoundary::Lower(n) => value.max(n),
            GaussBoundary::Upper(n) => value.min(n),
            GaussBoundary::ClampBoth((lower, upper)) => value.max(lower).min(upper),
            GaussBoundary::WrapBoth((lower, upper)) => {
                let mut wrapped = value;

                while wrapped < lower {
                    wrapped += -lower + upper;
                }
                while wrapped > upper {
                    wrapped += -upper + lower;
                }

                wrapped
            }
        }
    }
}

/// fn test_boundaries runs a series of test to ensure the proper functionality of the
/// gauss sampler boundary enforcement.
///
/// Samples each of the 4 boundary conditions 100 times and ensures enforcement on each sample
#[test]
fn test_boundaries() {
    let mut lower_g = Gauss::new(0.0, 1.0, GaussBoundary::Lower(0.0));
    let mut upper_g = Gauss::new(0.0, 1.0, GaussBoundary::Upper(0.0));
    let mut clamp_g = Gauss::new(0.0, 1.0, GaussBoundary::ClampBoth((-0.1, 0.1)));
    let mut wrap_g = Gauss::new(0.0, 1.0, GaussBoundary::WrapBoth((-0.1, 0.1)));

    for _ in 0..100 {
        assert!(lower_g.sample() >= 0.0);
        assert!(upper_g.sample() <= 0.0);
        assert!(clamp_g.sample() >= -0.1 && clamp_g.sample() <= 0.1);
        assert!(wrap_g.sample() >= -0.1 && wrap_g.sample() <= 0.1);
    }
}
