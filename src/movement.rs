use bevy::prelude::*;

use crate::tuning::Tuning;

#[derive(Component, Default)]
pub struct Velocity {
    pub linear: Vec2,
}

#[derive(Component, Default)]
pub struct Wrap;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

#[derive(Component, Default)]
pub struct Collider {
    pub radius: f32,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (integrate, wrap_positions).chain().in_set(MovementSet),
        );
    }
}

fn integrate(time: Res<Time>, mut movers: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_secs();
    for (mut transform, velocity) in &mut movers {
        transform.translation += (velocity.linear * dt).extend(0.0);
    }
}

fn wrap_positions(tuning: Res<Tuning>, mut wrappers: Query<&mut Transform, With<Wrap>>) {
    let half = tuning.playfield * 0.5;
    for mut transform in &mut wrappers {
        let p = &mut transform.translation;
        if p.x > half.x {
            p.x -= tuning.playfield.x;
        } else if p.x < -half.x {
            p.x += tuning.playfield.x;
        }
        if p.y > half.y {
            p.y -= tuning.playfield.y;
        } else if p.y < -half.y {
            p.y += tuning.playfield.y;
        }
    }
}
