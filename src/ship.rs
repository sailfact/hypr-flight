use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use crate::movement::{Velocity, Wrap};
use crate::shapes::ShapeAssets;
use crate::tuning::Tuning;

#[derive(Component)]
#[require(ShipIntent, Velocity, Wrap)]
pub struct Ship {
    pub cooldown: Timer,
}

#[derive(Component, Default)]
pub struct ShipIntent {
    pub turn: f32,
    pub thrusting: bool,
    pub firing: bool,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ShipFiring {
    entity: Entity,
    weapon: &'static mut Ship,
    intent: &'static mut ShipIntent,
    transform: &'static Transform,
    velocity: &'static Velocity,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShipSet;

#[derive(Component)]
struct ThrustFlame;

#[derive(Event)]
pub struct FireRequested {
    pub owner: Entity,
    pub origin: Vec2,
    pub direction: Dir2,
    pub inherited: Vec2,
}

/// Plugins
pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship)
            .add_systems(Update, (read_input, flame_visibility))
            .add_systems(FixedUpdate, (turn, thrust, fire).chain().in_set(ShipSet));
    }
}

fn spawn_ship(mut commands: Commands, shapes: Res<ShapeAssets>, tuning: Res<Tuning>) {
    commands.spawn((
        Ship {
            cooldown: ready_timer(tuning.fire_cooldown),
        },
        Mesh2d(shapes.ship.clone()),
        MeshMaterial2d(shapes.ship_material.clone()),
        Transform::default(),
        children![(
            ThrustFlame,
            Mesh2d(shapes.flame.clone()),
            MeshMaterial2d(shapes.flame_material.clone()),
            Transform::default(),
            Visibility::Hidden,
        )],
    ));
}

fn ready_timer(secs: f32) -> Timer {
    let mut t = Timer::from_seconds(secs, TimerMode::Once);
    let d = t.duration();
    t.tick(d);
    t
}

fn facing(transform: &Transform) -> Vec2 {
    (transform.rotation * Vec3::Y).truncate()
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, intent: Option<Single<&mut ShipIntent>>) {
    let Some(mut intent) = intent else { return };

    let left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);

    intent.turn = (left as i32 - right as i32) as f32;
    intent.thrusting = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);
    intent.firing |= keys.pressed(KeyCode::Space);
}

fn turn(
    time: Res<Time>,
    tuning: Res<Tuning>,
    ship: Option<Single<(&mut Transform, &ShipIntent), With<Ship>>>,
) {
    let Some(ship) = ship else { return };
    let (mut transform, intent) = ship.into_inner();
    transform.rotate_z(intent.turn * tuning.turn_rate * time.delta_secs());
}

fn thrust(
    time: Res<Time>,
    tuning: Res<Tuning>,
    ship: Option<Single<(&Transform, &ShipIntent, &mut Velocity)>>,
) {
    let Some(ship) = ship else { return };
    let (transform, intent, mut velocity) = ship.into_inner();
    let dt = time.delta_secs();

    if intent.thrusting {
        velocity.linear += facing(transform) * tuning.thrust * dt;
    }

    velocity.linear *= (-tuning.drag * dt).exp();
    velocity.linear = velocity.linear.clamp_length_max(tuning.max_speed);
}

fn flame_visibility(
    time: Res<Time>,
    intent: Option<Single<&ShipIntent>>,
    mut flames: Query<&mut Visibility, (With<ThrustFlame>, Without<Ship>)>,
) {
    let Some(intent) = intent else { return };
    let lit = intent.thrusting && (time.elapsed_secs() * 30.0).sin() > -0.3;

    for mut visibility in &mut flames {
        *visibility = if lit {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn fire(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut commands: Commands,
    ship: Option<Single<ShipFiring>>,
) {
    let Some(mut ship) = ship else { return };

    ship.weapon.cooldown.tick(time.delta());

    if !core::mem::take(&mut ship.intent.firing) || !ship.weapon.cooldown.is_finished() {
        return;
    }

    let heading = facing(ship.transform);
    let Ok(direction) = Dir2::new(heading) else {
        return;
    };
    ship.weapon.cooldown.reset();

    commands.trigger(FireRequested {
        owner: ship.entity,
        origin: ship.transform.translation.truncate() + heading * tuning.ship_radius * 1.8,
        direction,
        inherited: ship.velocity.linear,
    });
}
