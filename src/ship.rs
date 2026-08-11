use bevy::prelude::*;

use crate::movement::{Velocity, Wrap};
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

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tuning: Res<Tuning>,
) {
    let r = tuning.ship_radius;

    // Nose points along +Y. Bevy 2D treats +Y as up, and every facing
    // calculation later depends on this choice.
    let hull = meshes.add(Triangle2d::new(
        Vec2::new(0.0, r * 1.8),
        Vec2::new(-r, -r),
        Vec2::new(r, -r),
    ));
    let flame = meshes.add(Triangle2d::new(
        Vec2::new(0.0, -r * 2.2),
        Vec2::new(-r * 0.5, -r),
        Vec2::new(r * 0.5, -r),
    ));

    // Values above 1.0 are what drive bloom. This is not a mistake.
    let hull_mat = materials.add(Color::linear_rgb(0.7, 3.0, 4.5));
    let flame_mat = materials.add(Color::linear_rgb(5.0, 1.6, 0.3));

    commands.spawn((
        Ship {
            cooldown: ready_timer(tuning.fire_cooldown),
        },
        Mesh2d(hull),
        MeshMaterial2d(hull_mat),
        Transform::default(),
        children![(
            ThrustFlame,
            Mesh2d(flame),
            MeshMaterial2d(flame_mat),
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
    ship: Option<Single<(Entity, &mut Ship, &mut ShipIntent, &Transform, &Velocity)>>,
) {
    let Some(ship) = ship else { return };
    let (entity, mut ship, mut intent, transform, velocity) = ship.into_inner();

    ship.cooldown.tick(time.delta());

    if !core::mem::take(&mut intent.firing) || !ship.cooldown.is_finished() {
        return;
    }

    let heading = facing(transform);
    let Ok(direction) = Dir2::new(heading) else {
        return;
    };
    ship.cooldown.reset();

    commands.trigger(FireRequested {
        owner: entity,
        origin: transform.translation.truncate() + heading * tuning.ship_radius * 1.8,
        direction,
        inherited: velocity.linear,
    });
}
