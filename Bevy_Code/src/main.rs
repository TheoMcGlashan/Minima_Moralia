mod bodies;
mod camera;

use bevy::prelude::*;
use bodies::BodiesPlugin;
use camera::CameraPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BodiesPlugin)
        .add_plugins(CameraPlugin)
        .run();
}