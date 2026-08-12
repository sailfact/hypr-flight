use bevy::camera::Camera;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use crate::movement::Velocity;
use crate::shapes::ShapeAssets;
use crate::tuning::Tuning;

#[derive(Component)]
#[require(ShipIntent, Velocity)]
pub struct Ship {
    pub cooldown: Timer,
}

#[derive(Component, Default)]
pub struct ShipIntent {
    pub move_input: Vec2,
    pub aim_at: Option<Vec2>,
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

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ShipAim {
    transform: &'static mut Transform,
    intent: &'static ShipIntent,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShipSet;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ShipMotion {
    transform: &'static Transform,
    intent: &'static ShipIntent,
    velocity: &'static mut Velocity,
}

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
            .add_systems(FixedUpdate, (aim, thrust, fire).chain().in_set(ShipSet));
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

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    intent: Option<Single<&mut ShipIntent>>,
) {
    let Some(mut intent) = intent else { return };

    let (forward, back) = (keys.pressed(KeyCode::KeyW), keys.pressed(KeyCode::KeyS));
    let (left, right) = (keys.pressed(KeyCode::KeyA), keys.pressed(KeyCode::KeyD));

    intent.move_input = Vec2::new(
        (right as i32 - left as i32) as f32,
        (forward as i32 - back as i32) as f32,
    );

    let (camera, camera_transform) = *camera;

    // The letterbox viewport is offset from the window origin by the black bars,
    // but cursor_position() is window-relative. Subtract the offset.
    let vp_offset = camera
        .viewport
        .as_ref()
        .map(|v| v.physical_position.as_vec2())
        .unwrap_or(Vec2::ZERO);

    intent.aim_at = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world_2d(camera_transform, cursor - vp_offset)
            .ok()
    });

    intent.firing |= buttons.pressed(MouseButton::Left);
}

fn aim(ship: Option<Single<ShipAim>>) {
    let Some(mut ship) = ship else { return };

    let Some(target) = ship.intent.aim_at else {
        return;
    };
    let to_target = target - ship.transform.translation.truncate();
    let Ok(direction) = Dir2::new(to_target) else {
        return;
    };

    ship.transform.rotation =
        Quat::from_rotation_z(direction.to_angle() - std::f32::consts::FRAC_PI_2);
}

fn thrust(time: Res<Time>, tuning: Res<Tuning>, ship: Option<Single<ShipMotion>>) {
    let Some(mut ship) = ship else { return };
    let dt = time.delta_secs();

    if let Some(input) = ship.intent.move_input.try_normalize() {
        let world_dir = (ship.transform.rotation * input.extend(0.0)).truncate();
        ship.velocity.linear += world_dir * tuning.thrust * dt;
    }

    ship.velocity.linear *= (-tuning.drag * dt).exp();
    ship.velocity.linear = ship.velocity.linear.clamp_length_max(tuning.max_speed);
}

fn flame_visibility(
    time: Res<Time>,
    intent: Option<Single<&ShipIntent>>,
    mut flames: Query<&mut Visibility, (With<ThrustFlame>, Without<Ship>)>,
) {
    let Some(intent) = intent else { return };
    let lit = intent.move_input.y > 0.0 && (time.elapsed_secs() * 30.0).sin() > -0.3;

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
