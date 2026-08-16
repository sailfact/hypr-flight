#![allow(unused_imports)]
use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::background::BackgroundPlugin;
use crate::camera::CameraPlugin;

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
            BackgroundPlugin,
            CameraPlugin,
        ))
        .configure_sets(FixedUpdate, ship::ShipSet.before(movement::MovementSet))
        .run();
}

// fn letterbox(window: Single<&Window>, mut camera: Single<&mut Camera>, tuning: Res<Tuning>) {
//     let size = window.physical_size().as_vec2();
//     if size.x < 1.0 || size.y < 1.0 {
//         return; // minimised
//     }

//     let target = tuning.playfield.x / tuning.playfield.y;
//     let actual = size.x / size.y;

//     let (viewport_size, offset) = if actual > target {
//         let w = size.y * target;
//         (Vec2::new(w, size.y), Vec2::new((size.x - w) * 0.5, 0.0))
//     } else {
//         let h = size.x / target;
//         (Vec2::new(size.x, h), Vec2::new(0.0, (size.y - h) * 0.5))
//     };

//     camera.viewport = Some(Viewport {
//         physical_position: offset.as_uvec2(),
//         physical_size: viewport_size.as_uvec2(),
//         ..default()
//     });
// }
