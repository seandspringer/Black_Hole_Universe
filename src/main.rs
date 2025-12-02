use bevy::prelude::*;

mod objects;

/// Main Entry Point
///
/// simply starts up the event loop (App::new()...run())
/// and chains plugins. The body of this code all
/// exists as the `objects::BlackHoleUniverse` plugin.
/// See plugins.rs in the objects subdirectory
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(objects::BlackHoleUniverse)
        .run();
}
