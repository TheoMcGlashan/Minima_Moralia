use std::{f32::consts::FRAC_PI_2, ops::Range};
use bevy::{input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel}, math::ops::cbrt, prelude::*};

/// Camera settings for development purposes, will not change during runtime.
#[derive(Debug, Resource)]
struct CameraDevSettings {
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
    pub zoom_speed: f32,
    pub zoom_range: Range<f32>,
    pub move_speed: f32,
    pub pan_speed: f32,
}

/// Camera settings that can be modified during runtime.
#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub target: Vec3,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraDevSettings::default())
            .add_systems(Startup, (setup_camera, setup_ambient_light))
            .add_systems(Update, (orbit, zoom, move_camera, pan_camera));
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            orbit_distance: 20.0,
            target: Vec3::ZERO,
        }
    }
}

impl Default for CameraDevSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90 degress up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            pitch_speed: 0.0015,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.002,
            zoom_speed: 10.0,
            zoom_range: 5.0..100.0,
            move_speed: 10.,
            pan_speed: 0.5,
        }
    }
}

/// A function to increase brightness of the scene.
fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    println!("Setting up ambient light for the scene.");
    ambient_light.brightness = 500.0;
}

/// A system to spawn a camera with default settings.
fn setup_camera(
    mut commands: Commands,
    camera_settings: Res<CameraSettings>
) {
    commands.spawn((
        Name::new("Camera"),    // dev note: might not be necessary to have a name.
        Camera3d::default(),
        Transform::from_xyz(camera_settings.orbit_distance, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// A systen to orbit the camera around a point dependent on orbit distance.
fn orbit(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    camera_dev_settings: Res<CameraDevSettings>,
    camera_settings: Res<CameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_input.pressed(MouseButton::Right) {
        let delta = mouse_motion.delta;

        // No need to multiply by delta time as Accumulated Mouse Motion already accounts for it.
        let delta_pitch = delta.y * camera_dev_settings.pitch_speed;
        let delta_yaw = delta.x * camera_dev_settings.yaw_speed;

        // Obtain the existing pitch, yaw, and roll values from the transform.
        let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);

        // Establish the new yaw and pitch, preventing them from exceeding our limits.
        let pitch = (pitch - delta_pitch).clamp(
            camera_dev_settings.pitch_range.start,
            camera_dev_settings.pitch_range.end,
        );
        let yaw = yaw - delta_yaw;
        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    let target = camera_settings.target;
    camera_transform.translation = target - camera_transform.forward() * camera_settings.orbit_distance;
}

/// A system to change the orbit distance based on mouse wheel input.
fn zoom(
    mut evr_scroll: EventReader<MouseWheel>,
    camera_dev_settings: Res<CameraDevSettings>,
    mut camera_settings: ResMut<CameraSettings>
) {
    // Iterate through mouse wheel inputs and update the orbit distance accordingly.
    for ev in evr_scroll.read() {

        // Calculate the orbit distance as a value between 0.1 and 1 relative to the zoom range.
        let mut dist_modifier = camera_settings.orbit_distance / 
            (camera_dev_settings.zoom_range.end - camera_dev_settings.zoom_range.start);
        dist_modifier = dist_modifier.clamp(0.1, 1.0);

        // Adjust the orbit distance based on the scroll input and distance modifier.
        match ev.unit {
            MouseScrollUnit::Line =>{
                camera_settings.orbit_distance -= ev.y * camera_dev_settings.zoom_speed * dist_modifier;
            }
            // Pixel scroll is more precise, so we divide by 10 to make it less sensitive.
            MouseScrollUnit::Pixel => {
                camera_settings.orbit_distance -= ev.y * camera_dev_settings.zoom_speed * dist_modifier / 10.0;
            }
        }
        // Clamp the orbit distance to the defined zoom range.
        camera_settings.orbit_distance = camera_settings.orbit_distance.clamp(
            camera_dev_settings.zoom_range.start,
            camera_dev_settings.zoom_range.end,
        );
    }
}

/// A system to update the camera's target position based on button input.
fn move_camera(
    key_input: Res<ButtonInput<KeyCode>>,
    mut camera_settings: ResMut<CameraSettings>,
    camera_dev_settings: Res<CameraDevSettings>,
    camera_transform: Single<&Transform, With<Camera>>,
    time: Res<Time>,
) {
    let mut movement = Vec3::ZERO;

    // Update movement vector based on inputs.
    if key_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        movement -= *camera_transform.local_x(); // Move left.
    }
    if key_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        movement += *camera_transform.local_x(); // Move right.
    }
    if key_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        movement -= *camera_transform.local_z(); // Move forward.
    }
    if key_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        movement += *camera_transform.local_z(); // Move backward.
    }
    if key_input.any_pressed([KeyCode::Space, KeyCode::Enter]) {
        movement += *camera_transform.local_y(); // Move up.
    }
    if key_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        movement -= *camera_transform.local_y(); // Move down.
    }

    // Normalize movement and scale by delta time and orbit distance.
    if movement != Vec3::ZERO {
        movement = movement.normalize_or_zero() * time.delta_secs() * camera_dev_settings.move_speed 
            * cbrt(camera_settings.orbit_distance);
        camera_settings.target += movement;
    }
}

// A system to update the camera's target position based on mouse input.
fn pan_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut camera_settings: ResMut<CameraSettings>,
    camera_dev_settings: Res<CameraDevSettings>,
    camera_transform: Single<&Transform, With<Camera>>,
    time: Res<Time>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        let delta = mouse_motion.delta;

        // Calculate the movement vector based on the camera's local axes.
        let movement_up = delta.y * *camera_transform.local_y();
        let movement_right = -delta.x * *camera_transform.local_x();
        let movement = movement_up + movement_right;

        // Scale movement vector by delta time and pan speed, then apply to the camera target.
        camera_settings.target += movement * camera_dev_settings.pan_speed * time.delta_secs();
    }
}