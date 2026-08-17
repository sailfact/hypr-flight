use bevy::prelude::*;

use crate::tuning::Tuning;

//---------------------------------
// Components
//---------------------------------
#[derive(Component, Default)]
pub struct Velocity {
    pub linear: Vec2,
}

#[derive(Component, Default)]
pub struct Wrap;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

#[derive(Component, Default)]
#[allow(dead_code)]
pub struct Collider {
    // Used when we add physics
    pub radius: f32,
}

#[derive(Component, Default)]
pub struct Interp {
    prev: Vec2,
    cur: Vec2,
}

impl Interp {
    // Used when we add physics
    #[allow(dead_code)]
    pub fn at(position: Vec2) -> Self {
        Self {
            prev: position,
            cur: position,
        }
    }
}

//---------------------------------
// Plugins
//---------------------------------
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, interp_begin)
            .add_systems(
                FixedUpdate,
                (integrate, wrap_positions).chain().in_set(MovementSet),
            )
            .add_systems(FixedLast, interp_end)
            .add_systems(
                RunFixedMainLoop,
                interp_apply.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            );
    }
}

//---------------------------------
// Systems
//---------------------------------
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

/// Age `cur` into `prev`, then restore the authoritative position that
/// `interp_apply` overwrote for the renderer. Integrating from a blended
/// position instead would accumulate drift.
fn interp_begin(mut movers: Query<(&mut Transform, &mut Interp)>) {
    for (mut transform, mut interp) in &mut movers {
        interp.prev = interp.cur;
        transform.translation.x = interp.cur.x;
        transform.translation.y = interp.cur.y;
    }
}

/// Capture the result of the step now that integration (and later, wall
/// resolution) has run.
fn interp_end(mut movers: Query<(&Transform, &mut Interp)>) {
    for (transform, mut interp) in &mut movers {
        interp.cur = transform.translation.truncate();
    }
}

/// Blend between the last two fixed positions for this frame. On frames where
/// no fixed step ran, `overstep_fraction` has still advanced, so the entity
/// keeps gliding rather than freezing — which is what removes the staircase.
fn interp_apply(fixed: Res<Time<Fixed>>, mut movers: Query<(&mut Transform, &Interp)>) {
    let alpha = fixed.overstep_fraction();
    for (mut transform, interp) in &mut movers {
        let p = interp.prev.lerp(interp.cur, alpha);
        transform.translation.x = p.x;
        transform.translation.y = p.y;
    }
}
