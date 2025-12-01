use crate::objects::clocks::{BHCounter, TotalTime, WorldCounter, WorldTime};
use crate::objects::gamestate::{GameState, ThePlanet, UNIVERSE_SIZE};
use crate::objects::gauss::{Gauss, GaussBoundary};
use crate::objects::movables::{
    CollisionFrame, CollisionResult, CollisionSet, Movable, ObjectType, Velocity,
};
use crate::objects::sliders::{
    BLACKHOLE_COUNT_RNG, BLACKHOLE_MASS_RNG, BLACKHOLE_VEL_RNG, POSSTDEVMIN, SLIDERWIDTH,
    SliderBkg, SliderType, SliderValue, VELSTDEVMIN, generate_slider,
};
use crate::objects::traits::collisions::CollisionDetection;
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use std::collections::BTreeSet;
use std::sync::Mutex;

/// Bevy plugin definition
pub struct BlackHoleUniverse;

/// Implementation for the Bevy plugin: addes necessary
/// resources (similiar to globals),
/// required plugins (similiar to modules)
/// and registers systems (functions) to run
/// at every frame update. This is the heart of the program
impl Plugin for BlackHoleUniverse {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
        app.insert_resource(GameState::new());
        app.add_systems(Startup, (setup_field, setup_hub, setup_objects).chain());
        app.add_systems(
            Update,
            (drag_slider, update_slider, update_slider_results).chain(),
        );
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

/// not called directly from a system/event loop but is instead a helper function
/// called by either setup_objects or slider motion, etc to physically produce
/// a visual object on the playing field. All objects are Mesh2d circles where
/// only the color changes to distinguish between the objects
fn spawn_object(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    object: Movable,
) {
    let mut color = Color::linear_rgb(0.9, 0.9, 0.9);

    if object.otype == ObjectType::World {
        color = Color::linear_rgb(0.0, 0.9, 0.0);

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(object.size.radius))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(object.position.x, object.position.y, 0.0),
            ThePlanet,
            object,
        ));
    } else {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(object.size.radius))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(object.position.x, object.position.y, 0.0),
            object,
        ));
    }
}

/// A helper function like above, except removes an Entity (this is a
/// Bevy object / collection of components) from the game. Used to
/// destroy a visible object
fn destroy_object(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}

/// Schedule: Startup Bevy System
///
/// spawns the camera (2d) with its Orthographic Projection,
/// the space-time playing field
/// and a small red border to highlight the universe boundary against
/// the window background
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
    commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(UNIVERSE_SIZE - 10.0, UNIVERSE_SIZE - 10.0))),
            MeshMaterial2d(materials.add(Color::linear_rgb(0.0, 0.0, 0.0))),
            Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        ))
        .observe(place_planet)
        .observe(planet_dragged)
        .observe(check_for_start);

    //border
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(UNIVERSE_SIZE, UNIVERSE_SIZE))),
        MeshMaterial2d(materials.add(Color::linear_rgb(0.9, 0.3, 0.3))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
    ));
}

/// Schedule: Startup Bevy System
///
/// sets the initial state of the Universe (playing field)
/// all slider-bars default to 50% full and so the initial
/// configuration will represent this 50% option.
fn setup_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sliders: Query<(&SliderValue, &SliderType)>,
) {
    let mut bh_count = 0;
    let mut bh_mass = 0.0;
    let mut bh_vel = 0.0;
    let mut bh_pos_std = 0.0; //std for the position gauss

    let bh_mass_mean = (BLACKHOLE_MASS_RNG.upper + BLACKHOLE_MASS_RNG.lower) / 2.0;

    for (slider_value, slider_type) in sliders {
        match slider_type {
            SliderType::Count => {
                bh_count = (slider_value.value * BLACKHOLE_COUNT_RNG.upper as f32)
                    .max(BLACKHOLE_COUNT_RNG.lower as f32)
                    .round() as u32;
            }
            SliderType::Mass => {
                bh_mass = slider_value.value * bh_mass_mean;
            }
            SliderType::Velocity => {
                bh_vel = (slider_value.value + VELSTDEVMIN)
                    * (BLACKHOLE_VEL_RNG.upper.abs() + BLACKHOLE_VEL_RNG.lower.abs())
                    / 2.0;
            }
            SliderType::Density => {
                //use 1-slider value so that max on the bar squeezes the universe the most
                bh_pos_std = (1.0 - slider_value.value + POSSTDEVMIN) * UNIVERSE_SIZE / 2.0; //universesize/2 is max - basically fills the universe
            }
        }
    }

    let mut position_rand = Gauss::new(
        0.0,
        bh_pos_std,
        GaussBoundary::WrapBoth((-UNIVERSE_SIZE / 2.0, UNIVERSE_SIZE / 2.0)),
    );

    let mut bh_mass_rand = Gauss::new(
        bh_mass,
        BLACKHOLE_MASS_RNG.upper / 4.0,
        GaussBoundary::ClampBoth((BLACKHOLE_MASS_RNG.lower, BLACKHOLE_MASS_RNG.upper)),
    );

    let mut bh_vel_rand = Gauss::new(
        0.0,
        bh_vel,
        GaussBoundary::ClampBoth((BLACKHOLE_VEL_RNG.lower, BLACKHOLE_VEL_RNG.upper)),
    );

    for _ in 0..bh_count {
        spawn_object(
            &mut commands,
            &mut meshes,
            &mut materials,
            Movable::new(&ObjectType::BlackHole)
                .set_position(position_rand.sample(), position_rand.sample())
                .set_velocity(bh_vel_rand.sample(), bh_vel_rand.sample())
                .set_mass(bh_mass_rand.sample())
                .build(),
        );
    }
}

/// Schedule: Startup Bevy System
///
/// Bevy system which spawns the HUB: the
/// slider bar option controls, any visible text,
/// the progress timer counters, etc.
fn setup_hub(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    //spawn top left text: Total time and black hole counter
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(5),
            left: px(5),

            display: Display::Grid, // Use Grid display

            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)], // Two equal columns
            grid_template_rows: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)], // Two equal rows
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Total Time: "),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            ));
            parent.spawn((
                Text::new("0.00"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
                TotalTime,
            ));
            parent.spawn((
                Text::new("Black Holes: "),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            ));
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
                BHCounter,
            ));
        });

    // spawn top right text: World Time and Planets counter
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(5),
            right: px(-85), //Val::Percent(-5.0),

            display: Display::Grid, // Use Grid display

            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)], // Two equal columns
            grid_template_rows: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)], // Two equal rows
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("World Time: "),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            ));
            parent.spawn((
                Text::new("0.00"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
                WorldTime,
            ));
            parent.spawn((
                Text::new("Planets: "),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(0.5, 0.5, 0.0, 0.5)),
            ));
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::linear_rgba(1.0, 0.5, 0.0, 0.25)),
                WorldCounter,
            ));
        });

    let mut height_in_pixels = 1000;
    if let Ok(window) = window_query.single() {
        height_in_pixels = window.resolution.physical_height();
    }

    // spawn the Black Hole Settings group container and the title bar
    let left_container = commands
        .spawn((Node {
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
        },))
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

    //spawn blackhole count slider
    let count_slider = generate_slider(SliderType::Count, "Count");
    let count_base = commands
        .spawn((
            count_slider.base,
            Interaction::None,
            RelativeCursorPosition::default(),
            SliderValue::default(),
        ))
        .id();
    let count_bkg = commands.spawn((count_slider.bkg, SliderBkg)).id();
    let count_text = commands.spawn(count_slider.text).id();
    commands.entity(count_base).add_child(count_bkg);
    commands.entity(count_base).add_child(count_text);
    commands.entity(left_container).add_child(count_base);

    //spawn blackhole mass slider
    let mass_slider = generate_slider(SliderType::Mass, "Masses");
    let mass_base = commands
        .spawn((
            mass_slider.base,
            Interaction::None,
            RelativeCursorPosition::default(),
            SliderValue::default(),
        ))
        .id();
    let mass_bkg = commands.spawn((mass_slider.bkg, SliderBkg)).id();
    let mass_text = commands.spawn(mass_slider.text).id();
    commands.entity(mass_base).add_child(mass_bkg);
    commands.entity(mass_base).add_child(mass_text);
    commands.entity(left_container).add_child(mass_base);

    //spawn blackhole velocity slider
    let mass_slider = generate_slider(SliderType::Velocity, "Velocity");
    let mass_base = commands
        .spawn((
            mass_slider.base,
            Interaction::None,
            RelativeCursorPosition::default(),
            SliderValue::default(),
        ))
        .id();
    let mass_bkg = commands.spawn((mass_slider.bkg, SliderBkg)).id();
    let mass_text = commands.spawn(mass_slider.text).id();
    commands.entity(mass_base).add_child(mass_bkg);
    commands.entity(mass_base).add_child(mass_text);
    commands.entity(left_container).add_child(mass_base);

    //spawn blackhole density slider
    let mass_slider = generate_slider(SliderType::Density, "Density");
    let mass_base = commands
        .spawn((
            mass_slider.base,
            Interaction::None,
            RelativeCursorPosition::default(),
            SliderValue::default(),
        ))
        .id();
    let mass_bkg = commands.spawn((mass_slider.bkg, SliderBkg)).id();
    let mass_text = commands.spawn(mass_slider.text).id();
    commands.entity(mass_base).add_child(mass_bkg);
    commands.entity(mass_base).add_child(mass_text);
    commands.entity(left_container).add_child(mass_base);
}

/// Schedule: Update Bevy System
///
/// this system applies the changes made by the user
/// on any of the slider-bars to the real-time display.
/// Note: changes are only accepted prior to the start of the
/// game. Changes after the start immediately return from this system.
fn update_slider_results(
    state: Res<GameState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut objects: Query<(Entity, &mut Movable, &mut Transform), With<Movable>>,
    sliders: Query<(&SliderValue, &SliderType)>,
) {
    //only accept slider-changes prior to game start
    if state.game_started {
        return;
    }

    let mut count_difference: i32 = 0;
    let mut bh_mass = 0.0;
    let mut update_bh_masses = false;
    let mut bh_vel = 0.0;
    let mut update_bh_vel = false;
    let mut bh_pos_std = 0.0;
    let mut update_bh_pos = false;

    let bh_mass_mean = (BLACKHOLE_MASS_RNG.upper + BLACKHOLE_MASS_RNG.lower) / 2.0;

    //check all slider bars for changes and gather those changes, if neccessary
    for (slider_value, slider_type) in sliders {
        match slider_type {
            SliderType::Count => {
                let bh_count = slider_value;
                count_difference = (bh_count.value * BLACKHOLE_COUNT_RNG.upper as f32)
                    .max(BLACKHOLE_COUNT_RNG.lower as f32)
                    .round() as i32
                    - (bh_count.prev_value * BLACKHOLE_COUNT_RNG.upper as f32)
                        .max(BLACKHOLE_COUNT_RNG.lower as f32)
                        .round() as i32;
            }
            SliderType::Mass => {
                bh_mass = slider_value.value * bh_mass_mean;
                if slider_value.value != slider_value.prev_value {
                    update_bh_masses = true;
                }
            }
            SliderType::Velocity => {
                bh_vel = (slider_value.value + VELSTDEVMIN)
                    * (BLACKHOLE_VEL_RNG.upper.abs() + BLACKHOLE_VEL_RNG.lower.abs())
                    / 2.0;
                if slider_value.value != slider_value.prev_value {
                    update_bh_vel = true;
                }
            }
            SliderType::Density => {
                //use 1-slider value so that max on the bar squeezes the universe the most
                bh_pos_std = (1.0 - slider_value.value + POSSTDEVMIN) * UNIVERSE_SIZE / 2.0; //universesize/4 is max - basically fills the universe
                if slider_value.value != slider_value.prev_value {
                    update_bh_pos = true;
                }
            }
        }
    }

    // build our random-normal number generators using the slider-bar metrics from above:
    let mut position_rand = Gauss::new(
        0.0,
        bh_pos_std,
        GaussBoundary::WrapBoth((-UNIVERSE_SIZE / 2.0, UNIVERSE_SIZE / 2.0)),
    );

    let mut bh_mass_rand = Gauss::new(
        bh_mass,
        BLACKHOLE_MASS_RNG.upper / 4.0,
        GaussBoundary::ClampBoth((BLACKHOLE_MASS_RNG.lower, BLACKHOLE_MASS_RNG.upper)),
    );

    let mut bh_vel_rand = Gauss::new(
        0.0,
        bh_vel,
        GaussBoundary::ClampBoth((BLACKHOLE_VEL_RNG.lower, BLACKHOLE_VEL_RNG.upper)),
    );

    // if the blackhole masses slider has changed, implement those changes to objects already rendered:
    if update_bh_masses {
        for (_entity, mut movable, mut transform) in &mut objects {
            if movable.otype == ObjectType::BlackHole {
                let new_mass = bh_mass_rand.sample();
                let old_mass = movable.size.mass;
                let ratio = new_mass / old_mass;

                movable.set_mass(new_mass);
                transform.scale *= ratio;
            }
        }
    }

    // if the blackhole velocity slider changed, implement those changes to the objects already rendered:
    if update_bh_vel {
        for (_entity, mut movable, mut _transform) in &mut objects {
            if movable.otype == ObjectType::BlackHole {
                movable.set_velocity(bh_vel_rand.sample(), bh_vel_rand.sample());
            }
        }
    }

    // if the blackhole density slider changed, implement those changes to the objects already rendered:
    if update_bh_pos {
        for (_entity, mut movable, mut transform) in &mut objects {
            if movable.otype == ObjectType::BlackHole {
                let new_x = position_rand.sample();
                let new_y = position_rand.sample();
                let old_x = movable.position.x;
                let old_y = movable.position.y;
                let diff_x = new_x - old_x;
                let diff_y = new_y - old_y;

                movable.set_position(new_x, new_y);
                transform.translation.x += diff_x;
                transform.translation.y += diff_y;
            }
        }
    }

    //add any new objects as necessary
    while count_difference > 0 {
        spawn_object(
            &mut commands,
            &mut meshes,
            &mut materials,
            Movable::new(&ObjectType::BlackHole)
                .set_position(position_rand.sample(), position_rand.sample())
                .set_velocity(bh_vel_rand.sample(), bh_vel_rand.sample())
                .set_mass(bh_mass_rand.sample())
                .build(),
        );
        count_difference -= 1;
    }

    //remove objects, if necessary
    if count_difference < 0 {
        for (entity, movable, _transform) in &objects {
            if movable.otype == ObjectType::BlackHole {
                destroy_object(&mut commands, entity);
                count_difference += 1;
                if count_difference >= 0 {
                    break;
                }
            }
        }
    }
}

/// Schedule: Update Bevy System
///
/// checks if
///
/// 1. the mouse btn is pressed
/// 2. the mouse position is inside one of the slider-bars
/// 3. if 1 && 2, store the value inside the SliderValue struct
///    as a percentage of the bar [0-1]
///
/// this stored value will then be used in:
/// 1. fn update_slider to graphically show the slider bar change
/// 2. fn update_slider_results to apply these changes to the playing field
fn drag_slider(
    mut interaction_query: Query<(&Interaction, &RelativeCursorPosition, &mut SliderValue)>,
) {
    for (interaction, relative_cursor, mut slider_value) in &mut interaction_query {
        //check that mouse button is down
        if !matches!(*interaction, Interaction::Pressed) {
            continue;
        }

        //check that it was pressed inside the slider:
        let Some(pos) = relative_cursor.normalized else {
            continue;
        };

        //slider takes [0:1] but pos.x.clamp is [-0.5:0.5] so this works as expected:
        slider_value.prev_value = slider_value.value;
        slider_value.value = 0.5 + pos.x.clamp(-0.5, 0.5); //percentage
    }
}

/// Schedule: Update Bevy System
///
/// physically updates the background of the slider to give the movement response
/// by changing the width of the node containing the green-colored background
fn update_slider(
    parent_query: Query<(&Children, &SliderValue)>,
    mut child_query: Query<&mut Node, With<SliderBkg>>,
) {
    for (children, slider_value) in &parent_query {
        let mut bkg_iter = child_query.iter_many_mut(children);
        if let Some(mut node) = bkg_iter.fetch_next() {
            node.width = px(SLIDERWIDTH * slider_value.value);
        }
    }
}

/// Schedule: Update Bevy System
///
/// updates the text in the top-left (black hole) and top-right (planet)
/// timer boxes used to indicate the lifetime of the universe and of our
/// dear planet.
///
/// Note the odd format here:
/// - the query returns a Query<&mut Text> iterator
/// - the first deref returns the first &mut Text in that query return
/// - the second deref returns the internal &String struct underlying the Text
///   which we then alter via the format!(...) macro
fn update_clock(
    time: Res<Time>,
    mut total_time: Query<&mut Text, (With<TotalTime>, Without<WorldTime>)>,
    mut world_time: Query<&mut Text, (With<WorldTime>, Without<TotalTime>)>,
    state: Res<GameState>,
) {
    if state.game_started {
        if state.game_alive {
            for mut clock in &mut total_time {
                //First deref gets the Text object, 2nd gets the internal String
                **clock = format!("{:.2}", time.elapsed_secs_f64());
            }
        }

        if state.world_alive {
            for mut clock in &mut world_time {
                //First deref gets the Text object, 2nd gets the internal String
                **clock = format!("{:.2}", time.elapsed_secs_f64());
            }
        }
    }
}

/// Schedule: Update Bevy System
///
/// Updates the velocity of all objects on the playing field.
/// A vec of Velocity structs is built by calculting the new frame's
/// velocity using the time between frame renderings and then each
/// object's velocity is updated
fn update_velocity(
    time: Res<Time>,
    mut objects: Query<&mut Movable, With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_started && state.game_alive {
        let vec: Vec<&Movable> = objects.iter().collect();
        let mut velocities: Vec<Velocity> = Vec::new();

        for movable in &objects {
            velocities.push(movable.update_velocity(&vec, time.delta_secs()));
        }

        for (index, mut movable) in objects.iter_mut().enumerate() {
            movable.set_velocity(velocities[index].vx, velocities[index].vy);
        }
    }
}

/// Schedule: Update Bevy System
///
/// Physically moves the objects on the playing field.
/// uses the updated velocities as set by the above System and then
/// moves the objects based upon the frame rate. Note the wrap around
/// logic to enfource the Spherical Universe concept
fn update_motion(
    time: Res<Time>,
    mut objects: Query<(&mut Movable, &mut Transform), With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_started && state.game_alive {
        const BOUNDARY: f32 = 0.5 * UNIVERSE_SIZE;
        let elapsed = time.delta_secs();

        for (mut movable, mut transform) in &mut objects {
            //println!("{},{}", movable.velocity.vx, movable.velocity.vy);

            movable.position.x_prev = movable.position.x;
            movable.position.y_prev = movable.position.y;
            movable.update_location(elapsed);

            //spherical universe wrap around
            if movable.position.x > BOUNDARY {
                movable.position.x -= UNIVERSE_SIZE; //off to right
            } else if movable.position.x < -BOUNDARY {
                movable.position.x += UNIVERSE_SIZE; //off to left
            }
            if movable.position.y > BOUNDARY {
                movable.position.y -= UNIVERSE_SIZE; // off to top
            } else if movable.position.y < -BOUNDARY {
                movable.position.y += UNIVERSE_SIZE; //off to bottom
            }

            transform.translation.x = movable.position.x;
            transform.translation.y = movable.position.y;
        }
    }
}

/// Schedule: Update Bevy System
///
/// The workhorse of each frame:
/// iterates through each object and determines if the current object has
/// collided with another object.
///
/// Because this calculation in O(N^2) but is still embaressingly parallel,
/// the rayon iterator parallelization logic is used to calculate and collect
/// a CollisionTree in parallel.
///
/// 2 collection types are accumulated:
/// 1. to_despawn = BtreeSet<Entity>: Entities are id integer codes and so the BTreeSet
/// automatically guarantees that duplicates will be removed. Used for despawning objects from
/// the graphical display.
/// 2. to_destroy = CollisionFrame<'_>: see the movable.rs file for definition. In short, this is
/// a smart-struct used to prevent duplicate collisions and properly coallesce collision results
fn update_collisions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    objects: Query<(Entity, &mut Movable), With<Movable>>,
    state: Res<GameState>,
) {
    // next check for collisions
    if state.game_started && state.game_alive {
        //this set is designed so that the order of the two colliding objects doesn't matter
        //i.e. there will not be duplicates in this list

        //a lot of this complexity is to remove double counting and to handle group collisions
        //a group collision would be one where more than 2 items collided together within the last frame -
        //happens more often than one might think!

        let to_despawn: Mutex<BTreeSet<Entity>> = Mutex::new(BTreeSet::<Entity>::new());
        let to_destroy = Mutex::new(CollisionFrame::new());

        objects.par_iter().for_each(|(entity, movable)| {
            let mut set = CollisionSet::new();
            let mut collide = false;

            for (_, item) in objects.iter() {
                if item != movable && item.collided(movable) {
                    collide = true;
                    set.append(item);
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
            destroy_object(&mut commands, *item);
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

/// Schedule: Update Bevy System
///
/// The world requires a trigger to start the simulation. Here, it is a click onto the
/// Universe canvas, an (optional) drag, and a release. This system represents the logic
/// for the click whic will place the (heroic) planet at the location underneath the mouse
/// cursor.
///
/// Most of this confusing logic are just coordinate mappings: the trigger (On<Pointer<Press>>)
/// stores it's mouse coordinates in viewport coordinates = pixels of the rendering window but we
/// need World coordinates which represents the universe as seen by the camera
fn place_planet(
    trigger: On<Pointer<Press>>,
    mut state: ResMut<GameState>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    planet_query: Query<Entity, With<ThePlanet>>,
) {
    if state.game_started {
        return;
    }

    let position = trigger.pointer_location.position; //Vec2 in screen coordianates (int, top left is 0,0)

    let (camera, camera_transform) = *camera_query;
    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, position) {
        for entity in planet_query {
            //prevent any bugs with the click capture
            destroy_object(&mut commands, entity);
        }

        spawn_object(
            &mut commands,
            &mut meshes,
            &mut materials,
            Movable::new(&ObjectType::World)
                .set_position(world_pos.x, world_pos.y)
                .set_velocity(0.0, 0.0)
                .set_size(0.0, 50.0)
                .build(),
        );
        state.planet_placed = true;
    }
}

/// Schedule: Update Bevy System
///
/// This sytem captures the dragging motion after the initial click to
/// place the planet in the universe. The drag motion will change the velocity
/// such that the planet's heading will follow the arrow drawn from it's starting
/// location to the mouse's released location with the addition that the magnitude
/// of that distance is translated into velocity (kinda like strecthing a rubberband)
fn planet_dragged(
    drag: On<Pointer<Drag>>,
    state: Res<GameState>,
    mut planet_query: Query<&mut Movable, With<ThePlanet>>,
) {
    if state.game_started || planet_query.iter().len() == 0 {
        return;
    }

    let mut planet = planet_query.single_mut().unwrap();
    planet.velocity.vx += drag.delta.x * 10.0; //arb scaling that feels good
    planet.velocity.vy += -drag.delta.y * 10.0;
}

fn check_for_start(_trigger: On<Pointer<Release>>, mut state: ResMut<GameState>) {
    if state.game_started || !state.planet_placed {
        return;
    }

    state.game_started = true;
}

/// Schedule: Update Bevy System
///
/// Checks of end of game logic which occurs if only a single black hole remains (maximal
/// entropy in this universe). In this event, both clocks are guaranteed to stop and the
/// frame updating will also yeild.
///
/// This System also updates the black hole and planet counter Text graphics
fn check_for_gameover(
    objects: Query<(Entity, &Movable), With<Movable>>,
    mut bh_count_label: Query<&mut Text, (With<BHCounter>, Without<WorldCounter>)>,
    mut world_count_label: Query<&mut Text, (With<WorldCounter>, Without<BHCounter>)>,
    mut state: ResMut<GameState>,
) {
    let mut bh_count: usize = 0;
    let mut planet_count: usize = 0;

    for (_, movable) in objects {
        match movable.otype {
            ObjectType::BlackHole => bh_count += 1,
            ObjectType::World => planet_count += 1,
        }
    }

    if state.game_started {
        if planet_count == 0 {
            state.world_alive = false;
        }
        if bh_count + planet_count <= 1 {
            state.game_alive = false;
        }
    }

    //&Text -> Text -> String
    **bh_count_label.single_mut().unwrap() = format!("{}", bh_count);
    **world_count_label.single_mut().unwrap() = format!("{}", planet_count);
}
