use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::WindowResolution;

mod movement;
mod projectile;
mod shapes;
mod ship;
mod tuning;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "hypr-flight".into(),
                resolution: WindowResolution::new(1280, 720).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .add_systems(Startup, spawn_camera)
        .init_resource::<tuning::Tuning>()
        .add_plugins((
            shapes::ShapesPlugin,
            movement::MovementPlugin,
            ship::ShipPlugin,
            projectile::ProjectilePlugin,
        ))
        .configure_sets(FixedUpdate, ship::ShipSet.before(movement::MovementSet))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Bloom::default(),
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
    ));
}
