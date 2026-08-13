use bevy::camera::{ScalingMode, Viewport};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::state::{DisplayQuality, GameState, Volume, game, menu, splash};
use crate::tuning::Tuning;

mod movement;
mod projectile;
mod shapes;
mod ship;
mod state;
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
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .init_state::<GameState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, letterbox)
        .init_resource::<tuning::Tuning>()
        .add_plugins((
            shapes::ShapesPlugin,
            movement::MovementPlugin,
            ship::ShipPlugin,
            projectile::ProjectilePlugin,
            splash::splash_plugin,
            menu::menu_plugin,
            game::game_plugin,
        ))
        .configure_sets(FixedUpdate, ship::ShipSet.before(movement::MovementSet))
        .run();
}

fn spawn_camera(mut commands: Commands, tuning: Res<Tuning>) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: tuning.playfield.x,
                height: tuning.playfield.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        Bloom::default(),
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
    ));
}

fn letterbox(window: Single<&Window>, mut camera: Single<&mut Camera>, tuning: Res<Tuning>) {
    let size = window.physical_size().as_vec2();
    if size.x < 1.0 || size.y < 1.0 {
        return; // minimised
    }

    let target = tuning.playfield.x / tuning.playfield.y;
    let actual = size.x / size.y;

    let (viewport_size, offset) = if actual > target {
        let w = size.y * target;
        (Vec2::new(w, size.y), Vec2::new((size.x - w) * 0.5, 0.0))
    } else {
        let h = size.x / target;
        (Vec2::new(size.x, h), Vec2::new(0.0, (size.y - h) * 0.5))
    };

    camera.viewport = Some(Viewport {
        physical_position: offset.as_uvec2(),
        physical_size: viewport_size.as_uvec2(),
        ..default()
    });
}
