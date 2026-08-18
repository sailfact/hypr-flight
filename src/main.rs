use bevy::prelude::*;
use bevy::window::WindowResolution;

mod background;
mod camera;
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
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .init_resource::<tuning::Tuning>()
        .add_plugins((
            shapes::ShapesPlugin,
            movement::MovementPlugin,
            ship::ShipPlugin,
            projectile::ProjectilePlugin,
            background::BackgroundPlugin,
            camera::CameraPlugin,
        ))
        .configure_sets(FixedUpdate, ship::ShipSet.before(movement::MovementSet))
        .run();
}
