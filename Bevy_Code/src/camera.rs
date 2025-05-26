use std::{f32::consts::FRAC_PI_2, ops::Range};
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub roll_speed: f32,
    pub yaw_speed: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, orbit);
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90 degress up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 20.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            roll_speed: 1.0,
            yaw_speed: 0.004,
        }
    }
}

/// A system to spawn a camera with default settings.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),    // dev note: might not be necessary to have a name.
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.5, -30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    let delta = mouse_motion.delta;
    let mut delta_roll = 0.0;


    // Roll camera based on mouse button input.
    if mouse_buttons.pressed(MouseButton::Left) {
        delta_roll -= 1.0;
    }
    if mouse_buttons.pressed(MouseButton::Right) {
        delta_roll += 1.0;
    }

    // No need to multiply by delta time as Accumulated Mouse Motion already accounts for it.
    let delta_pitch = delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;

    // This is a button input, must multiply by delta time.
    delta_roll *- camera_settings.roll_speed * time.delta_secs();

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    // Establish the new yaw and pitch, preventing them from exceeding our limits.
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );
    let roll = roll + delta_roll;
    let yaw = yaw + delta_yaw;
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
}