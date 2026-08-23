use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use crate::asteroid::{Asteroid, spawn_asteroid};
use crate::level::Level;
use crate::movement::{Collider, Interp, Velocity};
use crate::physics::PhysicsSet;
use crate::projectile::Bullet;
use crate::shapes::ShapeAssets;
use crate::ship::Ship;
use crate::tuning::Tuning;

// Components
#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct ContactCooldown(pub Timer);

impl ContactCooldown {
    pub fn ready(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        let duration = timer.duration();
        timer.tick(duration);
        Self(timer)
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CombatSet;

// Queries
#[derive(QueryData)]
#[query_data(mutable)]
struct ShipBody {
    transform: &'static mut Transform,
    velocity: &'static mut Velocity,
    collider: &'static Collider,
    health: &'static mut Health,
    cooldown: &'static mut ContactCooldown,
}

#[derive(QueryData)]
struct DeadCandidate {
    entity: Entity,
    health: &'static Health,
    transform: &'static Transform,
    velocity: &'static Velocity,
    asteroid: Option<&'static Asteroid>,
    ship: Option<&'static Ship>,
}

// Plugin
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (bullet_damage, ship_vs_asteroids, despawn_dead)
                .chain()
                .in_set(CombatSet)
                .after(PhysicsSet),
        );
    }
}

// Systems
fn bullet_damage(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Collider, &Bullet)>,
    mut targets: Query<(Entity, &Transform, &Collider, &mut Health), Without<Bullet>>,
) {
    for (bullet_entity, bullet_transform, bullet_collider, bullet) in &bullets {
        let bullet_position = bullet_transform.translation.truncate();

        for (target_entity, target_transform, target_collider, mut health) in &mut targets {
            // A bullet never damages whoever fired it (spec section 10).
            if bullet.owner == target_entity {
                continue;
            }

            let reach = bullet_collider.radius + target_collider.radius;
            let target_position = target_transform.translation.truncate();
            if bullet_position.distance_squared(target_position) > reach * reach {
                continue;
            }

            health.0 = health.0.saturating_sub(1);
            commands.entity(bullet_entity).despawn();
            // One bullet, one target.
            break;
        }
    }
}

/// Ship-vs-asteroid: the ship takes damage and bounces, the asteroid is
/// unaffected. `physics.rs` is circle-vs-tile only, so the bounce is resolved
/// here — the asteroid acts as a moving wall that happens to hurt.
fn ship_vs_asteroids(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut ships: Query<
        (
            &mut Transform,
            &mut Velocity,
            &Collider,
            &mut Health,
            &mut ContactCooldown,
        ),
        With<Ship>,
    >,
    asteroids: Query<(&Transform, &Collider), (With<Asteroid>, Without<Ship>)>,
) {
    for (mut transform, mut velocity, collider, mut health, mut cooldown) in &mut ships {
        cooldown.0.tick(time.delta());

        let mut position = transform.translation.truncate();

        for (asteroid_transform, asteroid_collider) in &asteroids {
            let asteroid_position = asteroid_transform.translation.truncate();
            let reach = collider.radius + asteroid_collider.radius;

            let offset = position - asteroid_position;
            let distance_squared = offset.length_squared();
            if distance_squared >= reach * reach {
                continue;
            }

            // Push apart, then reflect — same shape as the wall resolution in
            // physics.rs, including the guard against adding energy to a body
            // already moving away.
            let distance = distance_squared.sqrt();
            let normal = if distance > 1e-4 {
                offset / distance
            } else {
                Vec2::Y
            };
            position += normal * (reach - distance);

            let closing = velocity.linear.dot(normal);
            if closing < 0.0 {
                velocity.linear -= (1.0 + tuning.wall_restitution) * closing * normal;
            }

            if cooldown.0.is_finished() {
                health.0 = health.0.saturating_sub(1);
                cooldown.0.reset();
            }
        }

        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Health at zero: asteroids split, the ship respawns at the level start.
fn despawn_dead(
    mut commands: Commands,
    shapes: Res<ShapeAssets>,
    tuning: Res<Tuning>,
    level: Res<Level>,
    dead: Query<(
        Entity,
        &Health,
        &Transform,
        &Velocity,
        Option<&Asteroid>,
        Option<&Ship>,
    )>,
) {
    for (entity, health, transform, velocity, asteroid, ship) in &dead {
        if health.0 > 0 {
            continue;
        }
        let position = transform.translation.truncate();

        if let Some(asteroid) = asteroid {
            if let Some(smaller) = asteroid.size.split() {
                // Two fragments, thrown perpendicular to the parent's travel so
                // they visibly separate instead of stacking.
                let sideways =
                    velocity.linear.perp().normalize_or(Vec2::X) * tuning.asteroid_split_speed;
                let gap = smaller.radius() * 1.1;

                for sign in [-1.0, 1.0] {
                    spawn_asteroid(
                        &mut commands,
                        &shapes,
                        smaller,
                        position + sideways.normalize_or(Vec2::X) * gap * sign,
                        velocity.linear + sideways * sign,
                    );
                }
            }
            commands.entity(entity).despawn();
            continue;
        }

        if ship.is_some() {
            // Spec section 10: respawn at the start point with no
            // invulnerability. Reusing the entity avoids rebuilding the child
            // hierarchy.
            let start = level.start_position();
            commands.entity(entity).insert((
                Health(tuning.player_health),
                Transform::from_translation(start.extend(0.0)),
                Interp::at(start),
                Velocity::default(),
            ));
            continue;
        }

        commands.entity(entity).despawn();
    }
}
