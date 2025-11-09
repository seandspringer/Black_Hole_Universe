use bevy::prelude::*;

mod objects;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(objects::BlackHoleUniverse)
        .run();
}
