use bevy::prelude::*;
use rand::RngExt;
// star_count   : density of stars in the layer
// parallax     : depth of the layer
// z            : draw order
// radius       : size of stars
// brightness   : intensity of the starts in the layer

const LAYERS: [(u32, f32, f32, f32, f32); 3] = [
    // star_count, parallax, z, radius, brightness
    (280, 0.12, -100.0, 1.2, 0.30),
    (180, 0.30, -99.0, 1.8, 0.55),
    (00, 0.55, -98.0, 2.6, 0.75),
];

const FIELD_SIZE: Vec2 = Vec2::new(2600.0, 1800.0);

#[derive(Component)]
struct Star {
    base: Vec2,
    parallax: f32,
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_starfield)
            // MUST run after the camera moves, or the field lags one frame and
            // shears visibly at speed. Order it explicitly against camera.rs.
            .add_systems(Update, wrap_stars /* .after(crate::camera::follow) */);
    }
}

fn spawn_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::rng();

    for (count, parallax, z, radius, brightness) in LAYERS {
        let mesh = meshes.add(Circle::new(radius));
        let material = materials.add(Color::srgb(brightness, brightness, brightness));

        for _ in 0..count {
            let base = Vec2::new(
                rng.random_range(-FIELD_SIZE.x * 0.5..FIELD_SIZE.x * 0.5),
                rng.random_range(-FIELD_SIZE.y * 0.5..FIELD_SIZE.y * 0.5),
            );

            commands.spawn((
                Star { base, parallax },
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(base.x, base.y, z),
            ));
        }
    }
}

fn wrap_stars(
    camera: Single<&Transform, (With<Camera2d>, Without<Star>)>,
    mut stars: Query<(&Star, &mut Transform), Without<Camera2d>>,
) {
    let cam = camera.translation.truncate();

    for (star, mut transform) in &mut stars {
        let apparent = star.base + cam * (1.0 - star.parallax);
        let rel = apparent - cam;

        transform.translation.x = cam.x + wrap(rel.x, FIELD_SIZE.x);
        transform.translation.y = cam.y + wrap(rel.y, FIELD_SIZE.y);
    }
}

/// Wrap `v` into `[-size/2, size/2)`.
fn wrap(v: f32, size: f32) -> f32 {
    (v + size * 0.5).rem_euclid(size) - size * 0.5
}
