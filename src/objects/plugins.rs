use bevy::camera::ScalingMode;
use bevy::camera::Viewport;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::objects::clocks::{TotalTime, WorldTime};
use crate::objects::gamestate::GameState;
use crate::objects::movables::{Movable, MovableTuple, ObjectType, Size, Velocity};

use crate::objects::traits::collisions::{CollisionDetection, Position, Shapes};

#[derive(Resource)]
pub struct BlackHoleCount(u32);
#[derive(Resource)]
pub struct WorldCount(u32);

const UNIVERSE_SIZE: f32 = 10_000.0f32;

pub struct BlackHoleUniverse;

impl Plugin for BlackHoleUniverse {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::new());
        app.insert_resource(BlackHoleCount(4));
        app.insert_resource(WorldCount(0));
        app.add_systems(Startup, (setup_objects, setup_hub));
        app.add_systems(
            Update,
            (
                update_clock,
                update_velocity,
                update_motion,
                update_collisions,
            ),
        );
    }
}

fn spawn_object(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    object: Movable,
) {
    let color = Color::linear_rgb(0.9, 0.9, 0.9);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(object.size.radius))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(object.position.x, object.position.y, 0.0),
        object,
    ));
}

fn setup_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    blackhole_total: Res<BlackHoleCount>,
    world_total: Res<WorldCount>,
) {
    let mut blackhole_count: u32 = 0;
    let mut world_count: u32 = 0;

    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: UNIVERSE_SIZE,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    //spawn the space-time
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(UNIVERSE_SIZE - 10.0, UNIVERSE_SIZE - 10.0))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.0, 0.0, 0.0))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
    ));

    //border
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(UNIVERSE_SIZE, UNIVERSE_SIZE))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.9, 0.3, 0.3))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
    ));

    //for i in 0..black_holes.0 {
    spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(blackhole_count, &ObjectType::BlackHole)
            .set_position(2500.0, -2500.0)
            .set_velocity(0.0, 1000.0)
            .set_mass(20.0)
            .build(),
    );
    /*
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(50.0))),
            MeshMaterial2d(materials.add(hole_color)),
            Transform::from_xyz(0.0, -5000.0, 0.0),
            Movable::new(blackhole_count, &ObjectType::BlackHole)
                .set_position(0.0, -5000.0)
                .set_velocity(0.0, 1000.0)
                .set_mass(20.0)
                .build(),
        ));
    */
    blackhole_count += 1;

    spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(blackhole_count, &ObjectType::BlackHole)
            .set_position(-2500.0, 2500.0)
            .set_velocity(0.0, -1000.0)
            .set_mass(10.0)
            .build(),
    );

    /*commands.spawn((
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(0.0, 5000.0, 0.0),
        Movable::new(blackhole_count, &ObjectType::BlackHole)
            .set_position(0.0, 5000.0)
            .set_velocity(0.0, -1000.0)
            .set_mass(10.0)
            .build(),
    ));*/

    blackhole_count += 1;

    /*commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(-5000.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(5000.0, 0.0, 0.0),
    ));
    */
    //}
}

fn setup_hub(mut commands: Commands) {
    commands
        .spawn((
            Text::new("Total Time: "),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                top: px(5),
                left: px(5),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
            TotalTime,
        ));

    commands
        .spawn((
            Text::new("World Time: "),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                top: px(5),
                right: px(5),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
            WorldTime,
        ));
}

fn update_clock(
    time: Res<Time>,
    mut total_time: Query<&mut TextSpan, (With<TotalTime>, Without<WorldTime>)>,
    mut world_time: Query<&mut TextSpan, (With<WorldTime>, Without<TotalTime>)>,
    state: Res<GameState>,
) {
    if state.game_alive {
        for mut clock in &mut total_time {
            //two dereferences get's handle to the Text child; the TextSpan object
            **clock = format!("{:.2}", time.elapsed_secs_f64());
        }
    }

    if state.world_alive {
        for mut clock in &mut world_time {
            //two dereferences get's handle to the Text child; the TextSpan object
            **clock = format!("{:.2}", time.elapsed_secs_f64());
        }
    }
}

fn update_velocity(
    time: Res<Time>,
    mut objects: Query<&mut Movable, With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_alive {
        let vec: Vec<&Movable> = objects.iter().collect();
        let mut velocities: Vec<Velocity> = Vec::new();
        // first update positions
        for movable in &objects {
            velocities.push(movable.update_velocity(&vec, time.delta_secs()));
        }

        for (index, mut movable) in objects.iter_mut().enumerate() {
            movable.set_velocity(velocities[index].vx, velocities[index].vy);
        }
    }
}

fn update_motion(
    time: Res<Time>,
    mut objects: Query<(&mut Movable, &mut Transform), With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_alive {
        // first update positions
        let BOUNDARY = 0.5 * UNIVERSE_SIZE;
        for (mut movable, mut transform) in &mut objects {
            //println!("{},{}", movable.velocity.vx, movable.velocity.vy);

            movable.position.x += movable.velocity.vx * time.delta_secs();
            movable.position.y += movable.velocity.vy * time.delta_secs();

            //spherical universe wrap around
            if movable.position.x > BOUNDARY {
                movable.position.x = movable.position.x - UNIVERSE_SIZE; //off to right
            } else if movable.position.x < -BOUNDARY {
                movable.position.x = UNIVERSE_SIZE + movable.position.x; //off to left
            }
            if movable.position.y > BOUNDARY {
                movable.position.y = movable.position.y - UNIVERSE_SIZE; // off to top
            } else if movable.position.y < -BOUNDARY {
                movable.position.y = UNIVERSE_SIZE + movable.position.y; //off to left
            }

            transform.translation.x = movable.position.x;
            transform.translation.y = movable.position.y;
        }
    }
}

fn update_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut objects: Query<(Entity, &mut Movable), With<Movable>>,
    state: Res<GameState>,
) {
    // next check for collisions
    if state.game_alive {
        //this set is designed so that the order of the two colliding objects doesn't matter
        //i.e. there will not be duplicates in this list
        let mut destroyed: BTreeSet<MovableTuple> = BTreeSet::<MovableTuple>::new();

        for (entity, movable) in objects.iter() {
            let mut found: BTreeSet<MovableTuple> = objects
                .iter()
                .filter(|x: &(Entity, &Movable)| {
                    if x.1 != movable {
                        x.1.collided(movable)
                    } else {
                        false
                    }
                })
                .map(|x: (Entity, &Movable)| {
                    commands.entity(x.0).despawn();
                    MovableTuple::new(x.1, movable)
                })
                .collect();
            if !found.is_empty() {
                destroyed.append(&mut found);
            }
        }

        for group in destroyed {
            println!("collision");
            let new = Movable::handle_collision(group.0, group.1);
            spawn_object(&mut commands, &mut meshes, &mut materials, new);
        }
    }
}
