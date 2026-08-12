use bevy::prelude::*;

use crate::movement::{Collider, Velocity, Wrap};
use crate::shapes::ShapeAssets;
use crate::ship::FireRequested;
use crate::tuning::Tuning;

#[derive(Component)]
#[require(Velocity, Wrap)]
pub struct Bullet {
    pub owner: Entity,
}

#[derive(Component)]
pub struct Lifetime(pub Timer);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_bullet)
            .add_systems(FixedUpdate, expire_lifetimes);
    }
}

fn spawn_bullet(
    fired: On<FireRequested>,
    mut commands: Commands,
    shapes: Res<ShapeAssets>,
    tuning: Res<Tuning>,
) {
    commands.spawn((
        Bullet { owner: fired.owner },
        Velocity {
            linear: fired.inherited + fired.direction * tuning.bullet_speed,
        },
        Collider {
            radius: tuning.bullet_radius,
        },
        Lifetime(Timer::from_seconds(tuning.bullet_lifetime, TimerMode::Once)),
        Mesh2d(shapes.bullet.clone()),
        MeshMaterial2d(shapes.bullet_material.clone()),
        Transform::from_translation(fired.origin.extend(0.0)),
    ));
}

fn expire_lifetimes(
    time: Res<Time>,
    mut commands: Commands,
    mut expiring: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut expiring {
        if lifetime.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
