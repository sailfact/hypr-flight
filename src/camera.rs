use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::movement::Velocity;
use crate::ship::*;
use crate::tuning::Tuning;

//--------------------------------------------
//Components & Resources
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

fn toggle_camera_debug() {
    todo!()
}
fn decay_trauma() {
    todo!()
}
fn follow_camera() {
    todo!()
}
