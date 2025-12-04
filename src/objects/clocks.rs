//! Clocks.rs
//!
//! the clocks module defines Bevy components that elements that update with every
//! frame and can be classified as counters

use bevy::prelude::*;

/// TotalTime struct: Component
///
/// Used for querying the Bevy Text entity which contains the total elapsed
/// time of the game. Total time will spin until only 1 object remains
#[derive(Component)]
pub struct TotalTime;

/// WorldTime struct: Component
///
/// Used for querying the Bevy Text entity which contains the total elapsed
/// time that the user placed planet has survived. Total time will spin until
/// the planet is destroyed by a blackhole
#[derive(Component)]
pub struct WorldTime;

/// BHCounter struct: Component
///
/// Used for querying the Bevy Text entity which contains the total number of
/// black holes currently in the universe.
#[derive(Component)]
pub struct BHCounter;

/// WorldCounter struct: Component
///
/// Used for querying the Bevy Text entity which contains the number of planets in the game.
/// This value will only be 0 or 1
#[derive(Component)]
pub struct WorldCounter;
