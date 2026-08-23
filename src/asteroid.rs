use bevy::prelude::*;
use rand::RngExt;

use crate::combat::Health;
use crate::level::Level;
use crate::movement::{Collider, Interp, Velocity, WallCollision};
use crate::physics::PhysicsSet;
use crate::shapes::ShapeAssets;
use crate::ship::Ship;
use crate::tuning::Tuning;

//---------------------------------
// Components
//---------------------------------

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AsteroidSize {
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    /// A 3-tile corridor is 96 units, so a centred asteroid leaves
    /// `48 - radius` per side and the ship needs 24 of it. Large at 18 leaves
    /// 30 — threadable, but it genuinely narrows the corridor, which is the
    /// interesting outcome. Anything at 24 or above is a cork.
    pub fn radius(self) -> f32 {
        match self {
            Self::Small => 20.0,
            Self::Medium => 60.0,
            Self::Large => 120.0,
        }
    }

    pub fn health(self) -> u32 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
        }
    }

    /// What this breaks into when destroyed. `None` means it just disappears.
    pub fn split(self) -> Option<AsteroidSize> {
        match self {
            Self::Small => None,
            Self::Medium => Some(Self::Small),
            Self::Large => Some(Self::Medium),
        }
    }
}

//---------------------------------
// Resources
//---------------------------------

/// Tiles with a clear 5x5 block around them — chamber interiors, never
/// corridors. A corridor spawn drops a rock into the player's path with no
/// line of sight and no reaction window, which is exactly the failure
/// success criterion 5 vetoes. Rocks reach corridors by drifting into them,
/// which the player can see coming.
#[derive(Resource)]
pub struct SpawnZones(Vec<Vec2>);

#[derive(Resource)]
pub struct WaveTimer(Timer);

//---------------------------------
// Plugin
//---------------------------------

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_spawn_zones)
            .add_systems(FixedUpdate, spawn_waves.before(PhysicsSet));
    }
}

//---------------------------------
// Spawning
//---------------------------------

/// Spawn one asteroid. Public because `combat.rs` calls it for the fragments
/// when a larger one is destroyed.
pub fn spawn_asteroid(
    commands: &mut Commands,
    shapes: &ShapeAssets,
    size: AsteroidSize,
    position: Vec2,
    velocity: Vec2,
) {
    let radius = size.radius();
    commands.spawn((
        Asteroid { size },
        Health(size.health()),
        Collider { radius },
        WallCollision,
        Velocity { linear: velocity },
        // Without this the rock judders: it moves at 64 Hz in FixedUpdate
        // while the display runs faster, and it spawns away from the origin.
        Interp::at(position),
        Mesh2d(shapes.asteroid.clone()),
        MeshMaterial2d(shapes.asteroid_material.clone()),
        Transform::from_translation(position.extend(0.0)).with_scale(Vec3::splat(radius)),
    ));
}

fn build_spawn_zones(mut commands: Commands, level: Res<Level>, tuning: Res<Tuning>) {
    let mut zones = Vec::new();
    for y in 0..level.height() as i32 {
        for x in 0..level.width() as i32 {
            let clear =
                (-2..=2).all(|dy| (-2..=2).all(|dx| !level.is_solid(IVec2::new(x + dx, y + dy))));
            if clear {
                zones.push(level.tile_center(IVec2::new(x, y)));
            }
        }
    }

    assert!(
        !zones.is_empty(),
        "no chamber tiles wide enough to spawn asteroids in"
    );

    commands.insert_resource(SpawnZones(zones));
    commands.insert_resource(WaveTimer(Timer::from_seconds(
        tuning.asteroid_wave_interval,
        TimerMode::Repeating,
    )));
}

/// Timed waves, spawning on a period regardless of how many are already alive.
fn spawn_waves(
    time: Res<Time>,
    tuning: Res<Tuning>,
    shapes: Res<ShapeAssets>,
    zones: Res<SpawnZones>,
    mut timer: ResMut<WaveTimer>,
    mut commands: Commands,
    ship: Query<&Transform, With<Ship>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let ship_position = ship.iter().next().map(|t| t.translation.truncate());
    let mut rng = rand::rng();

    for _ in 0..tuning.asteroid_wave_count {
        // A rock materialising next to the ship is unfair rather than hard, so
        // retry a few times before giving up on this one.
        let mut placed = None;
        for _ in 0..16 {
            let position = zones.0[rng.random_range(0..zones.0.len())];

            let too_close = ship_position.is_some_and(|ship| {
                position.distance_squared(ship)
                    < tuning.asteroid_spawn_clearance * tuning.asteroid_spawn_clearance
            });
            if !too_close {
                placed = Some(position);
                break;
            }
        }
        let Some(position) = placed else { continue };

        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(tuning.asteroid_min_speed..tuning.asteroid_max_speed);
        let velocity = Vec2::from_angle(angle) * speed;

        spawn_asteroid(
            &mut commands,
            &shapes,
            AsteroidSize::Large,
            position,
            velocity,
        );
    }
}
