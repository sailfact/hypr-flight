// ---------------------------------------------------------------------------
// Tuning fields this module expects (§11 additions)
// ---------------------------------------------------------------------------
//
//   camera_view_height     720.0   world units visible vertically at zoom 1.0
//                                  (≈ 22 tiles; see the margin note)
//   camera_follow_lambda     6.0   as specced
//   camera_zoom_lambda       3.0   NEW — deliberately slower than follow
//   camera_lookahead         0.35  seconds of current velocity
//   camera_lookahead_max   260.0   NEW — hard distance cap on the offset
//   camera_zoom_base         1.0   as specced
//   camera_zoom_max          1.35  as specced
//   shake_max_offset        18.0   world units at trauma 1.0
//   trauma_decay             1.4   trauma units per second
use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::movement::Velocity;
use crate::ship::Ship;
use crate::tuning::Tuning;

type ShipQuery<'w, 's> =
    Single<'w, 's, (&'static Transform, &'static Velocity), (With<Ship>, Without<FollowCamera>)>;

//--------------------------------------------
// Components & Resources
//--------------------------------------------

/// Smoothed camera state.
///
/// The follow position lives here rather than in `Transform` so that shake,
/// which *is* written to `Transform`, never feeds back into the smoothing.
/// Smoothing a Transform that already has shake in it low-passes the shake
/// into the follow and makes the camera wander.
#[derive(Component)]
pub struct FollowCamera {
    /// Smoothed follow position
    pub anchor: Vec2,
    /// Smoothered zoom factor
    pub zoom: f32,
    /// Snap instead of smoothing
    pub snap: bool,
    /// Debug toggles
    pub lookahead_enabled: bool,
    pub zoom_enabled: bool,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            anchor: Vec2::ZERO,
            zoom: 1.0,
            snap: true,
            lookahead_enabled: true,
            zoom_enabled: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct Trauma(pub f32);

impl Trauma {
    /// No caller yet — hits and explosions are what should feed this.
    #[allow(dead_code)]
    pub fn add(&mut self, amount: f32) {
        self.0 = (self.0 + amount).min(1.0);
    }
}

//--------------------------------------------
// Plugin
//--------------------------------------------

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Trauma>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (toggle_camera_debug, decay_trauma, follow_camera).chain(),
            );
    }
}

//--------------------------------------------
// Systems
//--------------------------------------------
fn spawn_camera(mut commands: Commands, tuning: Res<Tuning>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: tuning.camera_view_height,
            },
            scale: tuning.camera_zoom_base,
            ..OrthographicProjection::default_2d()
        }),
        FollowCamera {
            zoom: tuning.camera_zoom_base,
            ..default()
        },
    ));
}

/// Two keys to isolate the coupled parameters while tuning. §8 is right that
/// lookahead and zoom must be tuned together, but you still need to be able
/// to turn each off to find out which one is causing what.
fn toggle_camera_debug(keys: Res<ButtonInput<KeyCode>>, mut camera: Single<&mut FollowCamera>) {
    if keys.just_pressed(KeyCode::F1) {
        camera.lookahead_enabled = !camera.lookahead_enabled;
    }
    if keys.just_pressed(KeyCode::F2) {
        camera.zoom_enabled = !camera.zoom_enabled;
    }
}

fn decay_trauma(time: Res<Time>, tuning: Res<Tuning>, mut trauma: ResMut<Trauma>) {
    trauma.0 = (trauma.0 - tuning.trauma_decay * time.delta_secs()).max(0.0);
}

/// Runs in `Update`. `RunFixedMainLoop` completes before `Update`, so the
/// ship's `FixedUpdate` position is already final for this frame.
///
/// If the player entity does not exist (death pause), `Single` fails and the
/// system is skipped — the camera holds its last position, which is what you
/// want while waiting to respawn.
fn follow_camera(
    time: Res<Time>,
    tuning: Res<Tuning>,
    trauma: Res<Trauma>,
    player: ShipQuery,
    camera: Single<(&mut FollowCamera, &mut Transform, &mut Projection)>,
) {
    let dt = time.delta_secs();
    let (ship_transform, velocity) = player.into_inner();
    let (mut follow, mut transform, mut projection) = camera.into_inner();

    let ship = ship_transform.translation.truncate();
    let speed: f32 = velocity.linear.length();

    // --- Lookahead -------------------------------------------------------
    // Capped by distance, not by time. An uncapped time-based lookahead
    // scales with boost speed and pushes the ship toward the trailing edge
    // exactly when it is moving fastest — the setup for criterion 5.
    let lookahead = if follow.lookahead_enabled {
        (velocity.linear * tuning.camera_lookahead).clamp_length_max(tuning.camera_lookahead_max)
    } else {
        Vec2::ZERO
    };
    let target = ship + lookahead;

    // --- Zoom ------------------------------------------------------------
    // Normalised against boost speed, not max speed, so boosting keeps
    // widening the view instead of saturating the moment the cap is hit.
    let target_zoom: f32 = if follow.zoom_enabled {
        let t: f32 = (speed / tuning.boost_max_speed).clamp(0.0, 1.0);
        tuning.camera_zoom_base + (tuning.camera_zoom_max - tuning.camera_zoom_base) * t
    } else {
        tuning.camera_zoom_base
    };

    // --- Smoothing -------------------------------------------------------
    // 1 - exp(-lambda * dt) is framerate-independent; a fixed lerp factor
    // is not, and will change feel between a 60 Hz and a 144 Hz machine.
    if follow.snap {
        follow.anchor = target;
        follow.zoom = target_zoom;
        follow.snap = false;
    } else {
        let pos_alpha = 1.0 - (-tuning.camera_follow_lambda * dt).exp();
        follow.anchor = follow.anchor.lerp(target, pos_alpha);

        // Deliberately a separate, slower lambda. Sharing the follow lambda
        // makes every brake tap pump the view in and out.
        let zoom_alpha = 1.0 - (-tuning.camera_zoom_lambda * dt).exp();
        follow.zoom += (target_zoom - follow.zoom) * zoom_alpha;
    }

    // --- Output ----------------------------------------------------------
    // Shake is added on top and never stored back into `anchor`.
    let shake = shake_offset(trauma.0, tuning.shake_max_offset, time.elapsed_secs());
    let out = follow.anchor + shake;
    transform.translation.x = out.x;
    transform.translation.y = out.y;
    // z is left alone: Camera2d's default near plane depends on it.

    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale = follow.zoom;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn shake_offset(trauma: f32, max_offset: f32, elapsed: f32) -> Vec2 {
    if trauma <= 0.0 {
        return Vec2::ZERO;
    }
    // trauma² so that small trauma is nearly invisible and large trauma bites.
    let amount = trauma * trauma * max_offset;
    // Cheap deterministic wobble. Incommensurate frequencies so it does not
    // read as a repeating figure-of-eight. Swap for value noise if it buzzes.
    Vec2::new((elapsed * 37.0).sin(), (elapsed * 41.3).cos()) * amount
}

/// Call on respawn, or after any teleport, so the camera cuts rather than
/// flying across the level.
#[allow(dead_code)]
pub fn snap_camera(mut camera: Single<&mut FollowCamera>) {
    camera.snap = true;
}
