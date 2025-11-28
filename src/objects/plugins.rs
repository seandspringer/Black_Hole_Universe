use crate::objects::clocks::{BHCounter, TotalTime, WorldCounter, WorldTime};
use crate::objects::gamestate::GameState;
use crate::objects::gauss::{Gauss, GaussBoundary};
use crate::objects::movables::{
    CollisionFrame, CollisionResult, CollisionSet, Movable, ObjectType, Velocity,
};
use crate::objects::sliders::{
    BLACKHOLE_COUNT_RNG, BLACKHOLE_MASS_RNG, SLIDERWIDTH, SliderBkg, SliderGraphic, SliderType,
    SliderValue, generate_slider,
};
use crate::objects::traits::collisions::{CollisionDetection, Position, Shapes};
use bevy::camera::ScalingMode;
use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use rand::rng;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const UNIVERSE_SIZE: f32 = 50_000.0f32;

pub struct BlackHoleUniverse;

impl Plugin for BlackHoleUniverse {
    fn build(&self, app: &mut App) {
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
/// called by either setup_objects or slider motion, etc
fn spawn_object(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
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

fn destroy_object(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
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
    sliders: Query<(&SliderValue, &SliderType)>,
) {
    let mut bh_count = 0;
    let mut bh_mass = 0.0;
    let mut position_rand = Gauss::new(
        0.0,
        UNIVERSE_SIZE / 4.0,
        GaussBoundary::WrapBoth((-UNIVERSE_SIZE / 2.0, UNIVERSE_SIZE / 2.0)),
    );

    let bh_mass_mean = (BLACKHOLE_MASS_RNG.upper + BLACKHOLE_MASS_RNG.lower) / 2.0;

    for (slider_value, slider_type) in sliders {
        match slider_type {
            SliderType::BHCountSlider => {
                bh_count = (slider_value.value * BLACKHOLE_COUNT_RNG.upper as f32)
                    .max(BLACKHOLE_COUNT_RNG.lower as f32)
                    .round() as u32;

                println!("count = {}", bh_count);
            }
            SliderType::BHMassSlider => {
                bh_mass = slider_value.value * bh_mass_mean;
                println!("ave mass = {}", bh_mass);
            }
            _ => {}
        }
    }

    let mut bh_mass_rand = Gauss::new(
        bh_mass,
        BLACKHOLE_MASS_RNG.upper / 4.0,
        GaussBoundary::ClampBoth((BLACKHOLE_MASS_RNG.lower, BLACKHOLE_MASS_RNG.upper)),
    );

    for _ in 0..bh_count {
        spawn_object(
            &mut commands,
            &mut meshes,
            &mut materials,
            Movable::new(&ObjectType::BlackHole)
                .set_position(position_rand.sample(), position_rand.sample())
                .set_velocity(1200.0, 1000.0)
                .set_mass(bh_mass_rand.sample())
                .build(),
        );
    }

    //for i in 0..black_holes.0 {
    /*spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(2500.0, -2500.0)
            .set_velocity(1200.0, 1000.0)
            .set_mass(20.0)
            .build(),
    );*/

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

    /*spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(-2500.0, 2500.0)
            .set_velocity(-1200.0, -1000.0)
            .set_mass(21.0)
            .build(),
    );*/

    /*spawn_object(
        &mut commands,
        &mut meshes,
        &mut materials,
        Movable::new(&ObjectType::BlackHole)
            .set_position(0.0, 0.0)
            .set_velocity(0.0, 0.0)
            .set_mass(100.0)
            .build(),
    );*/

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
    //spawn top left text: Total time and black hole counter
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(5),
            left: px(5),

            //width: Val::Percent(10.0),
            //height: Val::Percent(10.0),
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
                //TextSpan::default(),
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
                //TextSpan::default(),
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

            //width: Val::Percent(20.0),
            //height: Val::Percent(10.0),
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
    /*
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
            SliderType::BHCountSlider,
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



    commands.entity(left_container).add_child(left_header);
    commands.entity(bh_count_slider).add_child(bh_count_bkg);
    commands.entity(bh_count_slider).add_child(bh_count_text);
    commands.entity(left_container).add_child(bh_count_slider);
    */

    commands.entity(left_container).add_child(left_header);

    //spawn blackhole count slider
    let count_slider = generate_slider(SliderType::BHCountSlider, "Count");
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
    let mass_slider = generate_slider(SliderType::BHMassSlider, "Masses");
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

fn update_slider_results(
    state: Res<GameState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut objects: Query<(Entity, &mut Movable, &mut Transform), With<Movable>>,
    sliders: Query<(&SliderValue, &SliderType)>,
) {
    if state.game_started {
        return;
    }

    let mut count_difference: i32 = 0;
    let mut bh_mass = 0.0;
    let mut update_bh_masses = false;

    let bh_mass_mean = (BLACKHOLE_MASS_RNG.upper + BLACKHOLE_MASS_RNG.lower) / 2.0;

    let mut position_rand = Gauss::new(
        0.0,
        UNIVERSE_SIZE / 4.0,
        GaussBoundary::WrapBoth((-UNIVERSE_SIZE / 2.0, UNIVERSE_SIZE / 2.0)),
    );

    //check black hole number for changes
    for (slider_value, slider_type) in sliders {
        match slider_type {
            SliderType::BHCountSlider => {
                let bh_count = slider_value;
                count_difference = (bh_count.value * BLACKHOLE_COUNT_RNG.upper as f32)
                    .max(BLACKHOLE_COUNT_RNG.lower as f32)
                    .round() as i32
                    - (bh_count.prev_value * BLACKHOLE_COUNT_RNG.upper as f32)
                        .max(BLACKHOLE_COUNT_RNG.lower as f32)
                        .round() as i32;
            }
            SliderType::BHMassSlider => {
                bh_mass = slider_value.value * bh_mass_mean;
                if slider_value.value != slider_value.prev_value {
                    update_bh_masses = true;
                }
            }
            _ => {}
        }
    }

    let mut bh_mass_rand = Gauss::new(
        bh_mass,
        BLACKHOLE_MASS_RNG.upper / 4.0,
        GaussBoundary::ClampBoth((BLACKHOLE_MASS_RNG.lower, BLACKHOLE_MASS_RNG.upper)),
    );

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

    while count_difference > 0 {
        //add
        spawn_object(
            &mut commands,
            &mut meshes,
            &mut materials,
            Movable::new(&ObjectType::BlackHole)
                .set_position(position_rand.sample(), position_rand.sample())
                .set_velocity(1200.0, 1000.0)
                .set_mass(bh_mass_rand.sample())
                .build(),
        );
        count_difference -= 1;
    }
    if count_difference < 0 {
        //remove
        
        for (entity, movable, _transform) in &objects {
            if movable.otype == ObjectType::BlackHole {
                    destroy_object(&mut commands, entity);
                    count_difference += 1;
                    if count_difference >= 0 {
                        break;
                    }
            }
        }
        
       /* let mut v: Vec<(Entity, &Movable, &Transform)> = objects.into_iter().collect();
        
        while count_difference < 0 {
            match v.pop() {
                Option::Some((entity, movable, _mesh)) => {
                    if movable.otype == ObjectType::BlackHole {
                        destroy_object(&mut commands, entity);
                        count_difference += 1;
                    }
                }
                _ => {}
            }
        }*/
    }
}

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

/// physically updates the background of the slider to give the movement response
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

fn update_velocity(
    time: Res<Time>,
    mut objects: Query<&mut Movable, With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_started && state.game_alive {
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
    if state.game_started && state.game_alive {
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
    if state.game_started && state.game_alive {
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
            destroy_object(&mut commands, *item);
            //commands.entity(*item).despawn();
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
    mut bh_count_label: Query<&mut Text, (With<BHCounter>, Without<WorldCounter>)>,
    mut world_count_label: Query<&mut Text, (With<WorldCounter>, Without<BHCounter>)>,
    mut state: ResMut<GameState>,
) {
    let item_count = objects.count();
    let mut bh_count: usize = 0;
    let mut planet_count: usize = 0;

    if state.game_started {
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

        //&Text -> Text -> String
        **bh_count_label.single_mut().unwrap() = format!("{}", bh_count);
        **world_count_label.single_mut().unwrap() = format!("{}", planet_count);
    }
}
