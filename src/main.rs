use bevy::camera::ScalingMode;
use bevy::camera::Viewport;
use bevy::prelude::*;

pub struct BlackHoleUniverse;

impl Plugin for BlackHoleUniverse {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::new());
        app.insert_resource(BlackHoleCount(2));
        app.add_systems(Startup, (setup_objects, setup_hub));
        app.add_systems(Update, (update_clock, update_motion));
    }
}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Velocity {
    vx: f32, //% of speed of light
    vy: f32,
}

#[derive(Component)]
struct Size {
    radius: f32, //radius = 3 * mass https://blackholes.stardate.org/resources/article-structure-of-a-black-hole.html
    mass: f32,   //solar masses = 1.989x10^30 Kg
}

#[derive(Component)]
struct Movable {
    position: Position,
    velocity: Velocity,
    size: Size,
}

#[derive(Component)]
struct TotalTime;

#[derive(Component)]
struct WorldTime;

#[derive(Resource)]
struct BlackHoleCount(u32);

const UNIVERSE_SIZE: f32 = 10_000.0f32;

fn setup_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    black_holes: Res<BlackHoleCount>,
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

    let hole_color = Color::linear_rgb(0.9, 0.9, 0.9);
    //for i in 0..black_holes.0 {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(0.0, -5000.0, 0.0),
        Movable {
            position: Position { x: 0.0, y: -5000.0 },
            velocity: Velocity {
                vx: 0.0,
                vy: 1000.0,
            },
            size: Size {
                radius: 50.0,
                mass: 20.0,
            },
        },
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(0.0, 5000.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(-5000.0, 0.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(hole_color)),
        Transform::from_xyz(5000.0, 0.0, 0.0),
    ));
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

#[derive(Resource)]
struct GameState {
    world_alive: bool,
    game_alive: bool,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            world_alive: true,
            game_alive: true,
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

fn update_motion(
    time: Res<Time>,
    mut objects: Query<(&mut Movable, &mut Transform), With<Movable>>,
    state: Res<GameState>,
) {
    if state.game_alive {
        for (mut movable, mut transform) in &mut objects {
            println!("{},{}", movable.velocity.vx, movable.velocity.vy);
            movable.position.x += movable.velocity.vx * time.delta_secs();
            movable.position.y += movable.velocity.vy * time.delta_secs();
            transform.translation.x += movable.velocity.vx * time.delta_secs();
            transform.translation.y += movable.velocity.vy * time.delta_secs();
        }
    }
}

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BlackHoleUniverse)
        .run();
}
