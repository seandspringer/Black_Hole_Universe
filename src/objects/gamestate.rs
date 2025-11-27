use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub world_alive: bool,
    pub game_alive: bool,
    pub game_started: bool,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world_alive: true,
            game_alive: true,
            game_started: false,
        }
    }
}
