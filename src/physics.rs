use bevy::prelude::*;

use crate::level::Level;
use crate::movement::{Collider, Velocity, WallCollision};
use crate::projectile::Bullet;
use crate::tuning::Tuning;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (step_bodies, despawn_bullets_in_walls).in_set(PhysicsSet),
        );
    }
}

/// How many equal substeps this frame's displacement must be split into so
/// that no single step moves further than half a tile (spec section 7.2). Splitting
/// this way makes tunnelling impossible without a continuous-collision system.
///
/// Always at least 1: a stationary body still needs one resolution pass, or it
/// stays wedged in whatever it was already overlapping.
pub fn substep_count(speed: f32, dt: f32, tile_size: f32) -> u32 {
    let displacement = speed * dt;
    let limit = tile_size * 0.5;
    ((displacement / limit).ceil() as u32).max(1)
}

/// Contact between a circle and one axis-aligned tile.
///
/// Returns `Some((normal, penetration))` where `normal` points from the tile
/// toward the circle centre, or `None` if they do not overlap.
pub fn circle_tile_overlap(
    level: &Level,
    tile: IVec2,
    centre: Vec2,
    radius: f32,
) -> Option<(Vec2, f32)> {
    let size = level.tile_size();
    let min = Vec2::new(tile.x as f32 * size, tile.y as f32 * size);
    let max = min + Vec2::splat(size);

    let closest = centre.clamp(min, max);
    let offset = centre - closest;
    let distance_squared = offset.length_squared();

    if distance_squared >= radius * radius {
        return None;
    }

    if distance_squared > 1e-8 {
        let distance = distance_squared.sqrt();
        return Some((offset / distance, radius - distance));
    }

    // The centre is inside the tile, so there is no closest-point direction to
    // normalise. Push out along whichever face is nearest. Without this branch
    // the normalisation above produces NaN and the entity leaves the world.
    let to_min = centre - min;
    let to_max = max - centre;

    let mut normal = Vec2::NEG_X;
    let mut depth = to_min.x;
    if to_max.x < depth {
        normal = Vec2::X;
        depth = to_max.x;
    }
    if to_min.y < depth {
        normal = Vec2::NEG_Y;
        depth = to_min.y;
    }
    if to_max.y < depth {
        normal = Vec2::Y;
        depth = to_max.y;
    }

    Some((normal, depth + radius))
}

/// Resolve a circle against every solid tile it currently overlaps.
///
/// Candidates are the tiles covering the circle's AABB — at most four for a
/// radius under one tile. Each is resolved in sequence, which is wrong in
/// corners and can jitter when wedged into a concave angle (spec section 7.2).
/// Accepted for the prototype.
pub fn resolve_at(
    level: &Level,
    mut position: Vec2,
    mut velocity: Vec2,
    radius: f32,
    restitution: f32,
    friction: f32,
) -> (Vec2, Vec2) {
    let min_tile = level.world_to_tile(position - Vec2::splat(radius));
    let max_tile = level.world_to_tile(position + Vec2::splat(radius));

    for y in min_tile.y..=max_tile.y {
        for x in min_tile.x..=max_tile.x {
            let tile = IVec2::new(x, y);
            if !level.is_solid(tile) {
                continue;
            }
            let Some((normal, penetration)) = circle_tile_overlap(level, tile, position, radius)
            else {
                continue;
            };

            position += normal * penetration;

            // Only reflect when actually travelling into the surface.
            // Unguarded, this adds energy on the second tile of a corner pair
            // and the ship accelerates by scraping along walls.
            let closing = velocity.dot(normal);
            if closing < 0.0 {
                velocity -= (1.0 + restitution) * closing * normal;
                let normal_part = velocity.dot(normal) * normal;
                let tangent = velocity - normal_part;
                velocity = normal_part + tangent * (1.0 - friction);
            }
        }
    }

    (position, velocity)
}

/// Advance one body through a whole fixed step, resolving after each substep.
pub fn step_body(
    level: &Level,
    mut position: Vec2,
    mut velocity: Vec2,
    radius: f32,
    dt: f32,
    restitution: f32,
    friction: f32,
) -> (Vec2, Vec2) {
    let steps = substep_count(velocity.length(), dt, level.tile_size());
    let sub_dt = dt / steps as f32;

    for _ in 0..steps {
        position += velocity * sub_dt;
        let resolved = resolve_at(level, position, velocity, radius, restitution, friction);
        position = resolved.0;
        velocity = resolved.1;
    }

    (position, velocity)
}

fn step_bodies(
    level: Res<Level>,
    tuning: Res<Tuning>,
    time: Res<Time>,
    mut bodies: Query<(&mut Transform, &mut Velocity, &Collider), With<WallCollision>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, collider) in &mut bodies {
        let (position, resolved) = step_body(
            &level,
            transform.translation.truncate(),
            velocity.linear,
            collider.radius,
            dt,
            tuning.wall_restitution,
            tuning.wall_friction,
        );
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        velocity.linear = resolved;
    }
}

/// Spec section 7.2: bullets integrate plainly and despawn on contact with any solid
/// tile, via a single point-in-tile test.
fn despawn_bullets_in_walls(
    mut commands: Commands,
    level: Res<Level>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &bullets {
        let tile = level.world_to_tile(transform.translation.truncate());
        if level.is_solid(tile) {
            commands.entity(entity).despawn();
        }
    }
}
