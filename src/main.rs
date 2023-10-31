use rand::Rng;

use bevy::{
    prelude::*,
    time::Time,
    window::{ WindowTheme, WindowResized },
    sprite::MaterialMesh2dBundle,
};
#[derive(Component, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component)]
struct Mass(f32);

#[derive(Component, Clone, Copy)]
struct Position(Vec2);

#[derive(Resource)]
struct WindowSize {
    x: f32,
    y: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        WindowSize { x: 800.0, y: 400.0 }
    }
}
#[derive(Resource)]
struct Configuration {
    num_molecules: u128,
    collision_detection_distance: f32,
}
#[derive(Resource)]
struct Stats {
    nb_hits: u128,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration { num_molecules: 1000, collision_detection_distance: 5.0 }
    }
}
#[derive(Resource)]
struct MoveTimer(Timer);
#[derive(Resource)]
struct StatsTimer(Timer);

// The Player object
#[derive(Component)]
struct Molecule;
// Signifies an object is collidable
#[derive(Component)]
struct Collider;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Gaz".into(),
                    resolution: (WindowSize::default().x, WindowSize::default().y).into(),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }),
            GazPlugin,
        ))
        .run();
}
pub struct GazPlugin;
impl Plugin for GazPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_system)
            // .add_systems(Update, print_position_system)
            .add_systems(Update, collision_system)
            .add_systems(Update, stats_system)
            .add_systems(Update, on_resize_system);
    }
}
fn stats_system(
    time: Res<Time>,
    mut timer: ResMut<StatsTimer>,
    mut stats: ResMut<Stats>,
    window_size: ResMut<WindowSize>
) {
    if timer.0.tick(time.delta()).just_finished() {
        println!(
            "stats - nb_hits/surface : {}",
            (stats.nb_hits as f32) * (window_size.x * window_size.y)
        );
        stats.nb_hits = 0;
    }
}
fn on_resize_system(
    mut q: Query<&mut Transform>,
    resize_event: Res<Events<WindowResized>>,
    mut window_size: ResMut<WindowSize>
) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        window_size.x = e.width;
        window_size.y = e.height;
    }
    for mut transform in q.iter_mut() {
        if transform.translation.y > window_size.y / 2.0 {
            transform.translation.y = window_size.y / 2.0;
        } else if transform.translation.y < -window_size.y / 2.0 {
            transform.translation.y = -window_size.y / 2.0;
        }
        if transform.translation.x > window_size.x / 2.0 {
            transform.translation.x = window_size.x / 2.0;
        } else if transform.translation.x < -window_size.x / 2.0 {
            transform.translation.x = -window_size.x / 2.0;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let mut rng = rand::thread_rng();

    commands.spawn(Camera2dBundle::default());
    for _ in 0..Configuration::default().num_molecules {
        // commands.spawn((
        //     SpriteBundle {
        //         texture: asset_server.load("icon.png"),
        //         transform: Transform::from_xyz(
        //             rng.gen_range(-WindowSize::default().x..WindowSize::default().x),
        //             rng.gen_range(-WindowSize::default().y..WindowSize::default().y),
        //             0.0
        //         ).with_scale(Vec3::new(0.05, 0.05, 0.05)),
        //         ..default()
        //     },
        //     Velocity(Vec2::new(rng.gen_range(0.0..150.0), rng.gen_range(0.0..150.0))),
        //     Molecule,
        // ));
        let color = ColorMaterial::from(
            Color::rgba(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                1.0
            )
        );

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(5.0).into()).into(),
                // material: materials.add(ColorMaterial::from(Color::PURPLE)),
                material: materials.add(color),
                transform: Transform::from_translation(
                    Vec3::new(
                        rng.gen_range(
                            -WindowSize::default().x / 2.0..WindowSize::default().x / 2.0
                        ),
                        rng.gen_range(
                            -WindowSize::default().y / 2.0..WindowSize::default().y / 2.0
                        ),
                        0.0
                    )
                ),
                ..default()
            },
            Molecule,
            Velocity(Vec2::new(rng.gen_range(0.0..200.0), rng.gen_range(0.0..200.0))),
        ));
    }
    commands.insert_resource(WindowSize::default());
    commands.insert_resource(Stats { nb_hits: 0 });
    commands.insert_resource(MoveTimer(Timer::from_seconds(0.01, TimerMode::Repeating)));
    commands.insert_resource(StatsTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
}

fn print_position_system(time: Res<Time>, mut timer: ResMut<MoveTimer>, query: Query<&Transform>) {
    if timer.0.tick(time.delta()).finished() {
        for transform in &query {
            println!("position: {} {}", transform.translation.x, transform.translation.y);
        }
    }
}

fn collision_system(mut q: Query<(&mut Transform, &mut Velocity)>) {
    let mut combinations = q.iter_combinations_mut();
    while let Some([mut a1, mut a2]) = combinations.fetch_next() {
        if
            a1.0.translation.distance(a2.0.translation) <
            Configuration::default().collision_detection_distance
        {
            // println!("boom");
            let v1 = a1.1.0.clone();
            let v2 = a2.1.0.clone();
            a2.1.0.x = -v1.x;
            a2.1.0.y = -v1.y;
            a1.1.0.x = -v2.x;
            a1.1.0.y = -v2.y;
        }
    }
}
fn move_system(
    time: Res<Time>,
    mut timer: ResMut<MoveTimer>,
    mut query: Query<(&mut Transform, &mut Velocity)>,
    window_size: ResMut<WindowSize>,
    mut stats: ResMut<Stats>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut transform, mut velocity) in &mut query {
            transform.translation.y += velocity.0.y * time.delta_seconds();
            transform.translation.x += velocity.0.x * time.delta_seconds();

            if transform.translation.y.abs() > window_size.y / 2.0 {
                velocity.0.y = -velocity.0.y;
                stats.nb_hits += 1;
            }
            if transform.translation.x.abs() > window_size.x / 2.0 {
                velocity.0.x = -velocity.0.x;
                stats.nb_hits += 1;
            }
        }
    }
}
