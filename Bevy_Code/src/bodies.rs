use bevy::prelude::*;
use bevy::math::FloatPow;
use rand::Rng;
use bevy::color::palettes::css::ORANGE_RED;

const GRAVITY_CONSTANT: f32 = 0.001;
const NUM_BODIES: usize = 160;

#[derive(Component, Default)]
struct Mass(f32);
#[derive(Component, Default)]
struct Acceleration(Vec3);
/// Last position used for Verlet integration.
#[derive(Component, Default)]
struct LastPos(Vec3);
#[derive(Component, Default)]
struct Star;

pub struct BodiesPlugin;

impl Plugin for BodiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, generate_bodies)
        .add_systems(FixedUpdate, (interact_bodies, integrate));
    }
}

/// A bundle for 3d objects with physics properties.
#[derive(Bundle, Default)]
struct BodyBundle {
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    mass: Mass,
    acceleration: Acceleration,
    last_pos: LastPos,
}

/// A function to generate a star and spherical bodies in random positions around the star.
fn generate_bodies(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // A sphere mesh for the bodies.
    let mesh = meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap());
    // Objects will have randomized colors and velocities chosen from these ranges.
    let color_range = 0.5..1.0;
    let vel_range = -0.5..0.5;

    let mut rng = rand::rng();
    // Iterate over the number of bodies to spawn.
    for _ in 0..NUM_BODIES {
        // Generate a random radius and mass for the body.
        let radius: f32 = rng.random_range(0.1..0.7);
        let mass_value = FloatPow::cubed(radius) * 10.;

        // Generate a random position for the body within a sphere of radius 15, with 
        // positions closer to the origin being more likely.
        let position = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        ).normalize()
            * ops::cbrt(rng.random_range(0.2f32..1.0))
            *15.;

        // Spawns a body with a random color and velocity, and a mass dependent on the radius.
        // Last position is set to a random position close to the current position.
        commands.spawn((
            BodyBundle {
                mesh: Mesh3d(mesh.clone()),
                material: MeshMaterial3d(materials.add(Color::srgb(
                    rng.random_range(color_range.clone()),
                    rng.random_range(color_range.clone()),
                    rng.random_range(color_range.clone()),
                ))),
                mass: Mass(mass_value),
                acceleration: Acceleration(Vec3::ZERO),
                last_pos: LastPos(
                    position -Vec3::new(
                        rng.random_range(vel_range.clone()),
                        rng.random_range(vel_range.clone()),
                        rng.random_range(vel_range.clone()),
                    ) * time.timestep().as_secs_f32(),
                ),
            },
            Transform {
                translation: position,
                scale: Vec3::splat(radius),
                ..default()
            },
        ));
    }

    // Spawn a large star at the origin with a bright orange-red color.
    let star_radius = 1.;
    commands
        .spawn((
            BodyBundle {
                mesh: Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap())),
                material: MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: ORANGE_RED.into(),
                    emissive: LinearRgba::from(ORANGE_RED) * 2.,
                    ..default()
                })),
                mass: Mass(500.),
                ..default()
            },
            Transform::from_scale(Vec3::splat(star_radius)),
            Star,
        ))
        .with_child(PointLight {
            color: Color::WHITE,
            range: 100.,
            radius: star_radius,
            ..default()
        });
}

/// A system to make each body respond to the gravity of the other bodies.
fn interact_bodies(mut query: Query<(&Mass, &GlobalTransform, &mut Acceleration, Option<&Star>)>) {
    // Iterate over all pairs of bodies.
    let mut iter = query.iter_combinations_mut();

    while let Some([(Mass(m1), transform1, mut acc1, star1), (Mass(m2), transform2, mut acc2, star2)]) = 
        iter.fetch_next()
    {
        // Vector between the two bodies.
        let delta = transform2.translation() - transform1.translation();
        let distance_sq: f32 = delta.length_squared();

        // Force between planets is inversely proportional to the square of the distance.
        let f = GRAVITY_CONSTANT / distance_sq;
        let force_unit_mass = delta * f;

        // Update the acceleration of each body based on the force exerted by the other.
        acc1.0 += force_unit_mass * *m2;
        acc2.0 -= force_unit_mass * *m1;

        // If either body is the star, reset it's acceleration to prevent it from moving.
        if let Some(_) = star1 {
            acc1.0 = Vec3::ZERO;
        } else if let Some(_) = star2 {
            acc2.0 = Vec3::ZERO;
        }
    }
}

/// A system to perform Verlet integration on the bodies.
fn integrate(time: Res<Time>, mut query: Query<(&mut Acceleration, &mut Transform, &mut LastPos)>) {
    let dt_sq = time.delta_secs() * time.delta_secs();

    // Iterate over each body to update its position.
    for (mut acc, mut transform, mut last_pos) in &mut query {

        // Formula for Verlet integration. Uses two positions instead of velocity and position to calculate
        // the next position of the body. Faster for GPUs to optimize than Euler integration.
        let new_pos = transform.translation * 2.0 - last_pos.0 + acc.0 * dt_sq;

        // Reset acceleration after applying it.
        acc.0 = Vec3::ZERO;
        
        // Update the last position to the current position.
        last_pos.0 = transform.translation;

        // Set the new position of the body.
        transform.translation = new_pos;
    }
}