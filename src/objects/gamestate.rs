//! Gametstate.rs
//!
//! This module defines overall gamestate parameters

use bevy::prelude::*;

/// The width and height of the universe grid. Used
/// to devide the world into grid points as well as set the
/// orthographic projection of the camera
pub const UNIVERSE_SIZE: f32 = 25_000.0f32;

/// ThePlanet struct: Component
///
/// Bevy component for tracking and querying the user-placed
/// planet into the universe
#[derive(Component)]
pub struct ThePlanet;

/// GameState struct: Resource
///
/// GameState contains the overall state of the simulation. Because this
/// simulation is rather simple, 4 booleans completley define all states of the game
/// 1. world_alive - the user placed world is still active in the simulation
/// 2. game_alive - at least 2 objects remain in the universe
/// 3. game_started - user must place a planet and flick it to give it velocity to start simulation
/// 4. planet_placed - once user places planet, the flick motion will be captured to give it velocity
#[derive(Resource)]
pub struct GameState {
    pub world_alive: bool,
    pub game_alive: bool,
    pub game_started: bool,
    pub planet_placed: bool,
}

/// Standard constructor provide only which defaults to the pre-started game state
impl GameState {
    pub fn new() -> Self {
        GameState {
            world_alive: true,
            game_alive: true,
            game_started: false,
            planet_placed: false,
        }
    }
}
