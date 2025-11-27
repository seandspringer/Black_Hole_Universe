use crate::objects::clocks::{TotalTime, WorldTime};
use crate::objects::gamestate::GameState;
use crate::objects::movables::{
    CollisionFrame, CollisionResult, CollisionSet, Movable, ObjectType, Velocity,
};
use crate::objects::traits::collisions::{CollisionDetection, Position, Shapes};
use bevy::camera::ScalingMode;
use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const UNIVERSE_SIZE: f32 = 100_000.0f32;

pub struct BlackHoleUniverse;

impl Plugin for BlackHoleUniverse {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::new());
        app.add_systems(Startup, (setup_field, setup_hub, setup_objects).chain());
        app.add_systems(Update, (drag_slider, update_slider).chain());
        app.add_systems(
            Update,
            (
                update_clock,
                update_velocity,
                update_motion,
                update_collisions,
                check_for_gameover,
            )
                .chain(),
        );
    }
}

#[derive(Component)]

struct SliderValue {
    value: f32,
}
impl Default for SliderValue {
    fn default() -> Self {
        SliderValue { value: 0.5 }
    }
}
const SLIDERWIDTH: f32 = 100.0;

#[derive(Component)]
struct SliderBkg;

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

fn setup_field(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
}

fn setup_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //for i in 0..black_holes.0 {
    spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(2500.0, -2500.0)
            .set_velocity(1200.0, 1000.0)
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

    spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(-2500.0, 2500.0)
            .set_velocity(-1200.0, -1000.0)
            .set_mass(21.0)
            .build(),
    );

    spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(0.0, 0.0)
            .set_velocity(0.0, 0.0)
            .set_mass(100.0)
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

fn setup_hub(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
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

    let mut height_in_pixels = 1000;
    if let Ok(window) = window_query.single() {
        height_in_pixels = window.resolution.physical_height();
        println!(
            "{} {}",
            window.resolution.physical_height(),
            window.resolution.physical_width()
        )
    }

    let left_container = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column, // Stack children vertically
                row_gap: Val::Px(15.0),
                top: px(50),
                left: px(20),
                height: px(height_in_pixels - 200),
                width: px(SLIDERWIDTH * 2.0),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            //BorderColor::all(Color::WHITE),
            //Outline::new(px(1), Val::ZERO, Color::WHITE),
        ))
        .id();

    let bh_count_text = commands
        .spawn((
            Text::new("Count"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
        ))
        .id();

    let bh_count_slider = commands
        .spawn((
            Node {
                //position_type: PositionType::Absolute,
                height: px(50.0),
                width: px(SLIDERWIDTH),
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            Outline::new(px(1), Val::ZERO, Color::WHITE),
            Interaction::None,
            RelativeCursorPosition::default(),
            SliderValue::default(),
        ))
        .id();

    let bh_count_bkg = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                height: px(50.0),
                width: px(SLIDERWIDTH / 2.0),
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.4, 0.0, 1.0)),
            SliderBkg,
        ))
        .id();

    let left_header = commands
        .spawn((
            Text::new("Black Hole Settings"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::linear_rgba(0.9, 0.9, 0.9, 0.5)),
        ))
        .id();

    commands.entity(left_container).add_child(left_header);
    commands.entity(bh_count_slider).add_child(bh_count_bkg);
    commands.entity(bh_count_slider).add_child(bh_count_text);
    commands.entity(left_container).add_child(bh_count_slider);
}

fn drag_slider(
    mut interaction_query: Query<(&Interaction, &RelativeCursorPosition, &mut SliderValue)>,
) {
    for (interaction, relative_cursor, mut slider_value) in &mut interaction_query {
        if !matches!(*interaction, Interaction::Pressed) {
            continue;
        }

        let Some(pos) = relative_cursor.normalized else {
            continue;
        };

        slider_value.value = 0.5 + pos.x.clamp(-0.5, 0.5);
    }
}

fn update_slider(
    parent_query: Query<(&Children, &SliderValue)>,
    mut child_query: Query<&mut Node, With<SliderBkg>>,
) {
    for (children, slider_value) in &parent_query {
        let mut bkg_iter = child_query.iter_many_mut(children);
        if let Some(mut node) = bkg_iter.fetch_next() {
            // All nodes are the same width, so `NODE_RECTS[0]` is as good as any other.
            node.width = px(SLIDERWIDTH * slider_value.value);
        }
    }
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
        const BOUNDARY: f32 = 0.5 * UNIVERSE_SIZE;
        let elapsed = time.delta_secs();

        for (mut movable, mut transform) in &mut objects {
            //println!("{},{}", movable.velocity.vx, movable.velocity.vy);

            movable.position.x_prev = movable.position.x;
            movable.position.y_prev = movable.position.y;
            movable.update_location(elapsed);

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
    objects: Query<(Entity, &mut Movable), With<Movable>>,
    state: Res<GameState>,
) {
    // next check for collisions
    if state.game_alive {
        //this set is designed so that the order of the two colliding objects doesn't matter
        //i.e. there will not be duplicates in this list

        //a lot of this complexity is to remove double counting and to handle group collisions
        //a group collision would be one were more than 2 items collided together within the last frame

        let to_despawn: Mutex<BTreeSet<Entity>> = Mutex::new(BTreeSet::<Entity>::new());
        let to_destroy = Mutex::new(CollisionFrame::new());

        objects.par_iter().for_each(|(entity, movable)| {
            let mut set = CollisionSet::new();
            let mut collide = false;

            for (_, item) in objects.iter() {
                if item != movable {
                    if item.collided(movable) {
                        collide = true;
                        set.append(item);
                    }
                }
            }

            if collide {
                let mut despawn_lock = to_despawn.lock().unwrap();
                despawn_lock.insert(entity);

                set.append(movable);

                let mut destroy_lock = to_destroy.lock().unwrap();
                destroy_lock.push(set);
            }
        });

        let to_despawn = to_despawn.lock().unwrap();
        for item in to_despawn.iter() {
            commands.entity(*item).despawn();
        }

        match to_destroy.lock().unwrap().collect() {
            CollisionResult::Single(n) => {
                spawn_object(&mut commands, &mut meshes, &mut materials, n);
            }
            CollisionResult::NSize(n) => {
                //then add
                for new in n {
                    spawn_object(&mut commands, &mut meshes, &mut materials, new);
                }
            }
            _ => {}
        }
    }
}

fn check_for_gameover(
    objects: Query<(Entity, &Movable), With<Movable>>,
    mut state: ResMut<GameState>,
) {
    let item_count = objects.count();
    let mut bh_count: usize = 0;
    let mut planet_count: usize = 0;

    if item_count <= 1 {
        state.world_alive = false;
        state.game_alive = false;
    } else {
        for (_, movable) in objects {
            match movable.otype {
                ObjectType::BlackHole => bh_count += 1,
                ObjectType::World => planet_count += 1,
                _ => {}
            }
        }

        if planet_count == 0 {
            state.world_alive = false;
        }
        if bh_count == 1 {
            state.game_alive = false;
        }
    }
}
