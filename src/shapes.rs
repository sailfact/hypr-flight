use bevy::prelude::*;

use crate::level;
use crate::svg_mesh::svg_to_mesh;
use crate::tuning::Tuning;

#[derive(Resource)]
pub struct ShapeAssets {
    pub asteroid: Handle<Mesh>,
    pub asteroid_material: Handle<ColorMaterial>,
    pub ship: Handle<Mesh>,
    pub flame: Handle<Mesh>,
    pub bullet: Handle<Mesh>,
    pub ship_material: Handle<ColorMaterial>,
    pub flame_material: Handle<ColorMaterial>,
    pub bullet_material: Handle<ColorMaterial>,
    pub tile_mesh: Handle<Mesh>,
    pub tile_material: Handle<ColorMaterial>,
}

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, build_shapes);
    }
}

fn build_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tuning: Res<Tuning>,
) {
    let r = tuning.ship_radius;
    commands.insert_resource(ShapeAssets {
        ship: meshes.add(svg_to_mesh(
            include_str!("../assets/ships/player.svg"),
            r * 2.0 * 2.5,
        )),
        flame: meshes.add(Triangle2d::new(
            Vec2::new(0.0, -r * 2.2),
            Vec2::new(-r * 0.5, -r),
            Vec2::new(r * 0.5, -r),
        )),
        bullet: meshes.add(Circle::new(tuning.bullet_radius)),
        ship_material: materials.add(Color::linear_rgb(0.7, 3.0, 4.5)),
        flame_material: materials.add(Color::linear_rgb(5.0, 1.6, 0.3)),
        bullet_material: materials.add(Color::linear_rgb(6.0, 4.5, 2.0)),
        tile_mesh: meshes.add(Rectangle::new(level::TILE_SIZE, level::TILE_SIZE)),
        tile_material: materials.add(Color::linear_rgb(0.10, 0.12, 0.18)),
        asteroid: meshes.add(svg_to_mesh(
            include_str!("../assets/asteroids/medium_asteroid.svg"),
            2.0,
        )),
        asteroid_material: materials.add(Color::linear_rgb(0.35, 0.33, 0.30)),
    });
}
