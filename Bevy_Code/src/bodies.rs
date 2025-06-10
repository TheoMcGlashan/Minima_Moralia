use bevy::prelude::*;
use bevy::math::FloatPow;
use rand::Rng;

const GRAVITY: f32 = 5.;
const REPULSION: f32 = 2.;
const NUM_BODIES: usize = 165;
const DAMPING: f32 = 0.005;

#[derive(Component, Default)]
struct Mass(f32);
#[derive(Component, Default)]
struct Acceleration(Vec3);
/// Last position used for Verlet integration.
#[derive(Component, Default)]
struct LastPos(Vec3);
#[derive(Component, Default)]
struct Radius(f32);

pub struct BodiesPlugin;

impl Plugin for BodiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (generate_bodies, setup_ambient_light))
        .add_systems(FixedUpdate, (sphere_repulsion, integrate, gravity));
    }
}

/// A bundle for 3d objects with physics properties.
#[derive(Bundle, Default)]
struct BodyBundle {
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
    mass: Mass,
    radius: Radius,
    acceleration: Acceleration,
    last_pos: LastPos,
}

/// A function to increase brightness of the scene.
fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    println!("Setting up ambient light for the scene.");
    ambient_light.brightness = 500.0;
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
        let radius: f32 = rng.random_range(1.0..2.0);
        let mass_value = FloatPow::cubed(radius) * 0.1;

        // Generate a random position for the body within a sphere of radius 15, with 
        // positions closer to the origin being more likely.
        let position = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        ).normalize()
            * ops::cbrt(rng.random_range(0.2f32..1.0))
            *50.;

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
                radius: Radius(radius),
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
}

/// A system to make each body respond to the gravity of the other bodies.
fn sphere_repulsion(mut query: Query<(&Mass, &Radius, &GlobalTransform, &mut Acceleration)>) {
    // Iterate over all pairs of bodies.
    let mut iter = query.iter_combinations_mut();

    while let Some([(Mass(m1), Radius(r1), transform1, mut acc1), (Mass(m2), Radius(r2), transform2, mut acc2)]) = 
        iter.fetch_next()
    {
        // Vector between bodies.
        let delta = transform2.translation() - transform1.translation();
        // Scale our force by the size of the bodies, so larger bodies push more.
        let r_sum = r1 + r2;
        let r_distance = delta.length() / r_sum;

        // Force between bodies is inversely proportional to their distance apart.
        let magnitude = REPULSION / r_distance.squared();
        let force: Vec3 = delta * magnitude;

        // Apply the force to both bodies. Bodies repel each other.
        acc1.0 -= force * *m2;
        acc2.0 += force * *m1;
    }
}

/// A system to apply gravity to bodies.
fn gravity(mut query: Query<(&Mass, &GlobalTransform, &mut Acceleration)>
) {
    for (mass, transform, mut acceleration) in &mut query {

        // Calculate the force of gravity based on the distance from the origin.
        let force = GRAVITY * mass.0 + 
            (transform.translation().length() / 10.).powf(2.0);

        // If the force is too small, skip applying it to avoid numerical instability.
        if force < 0.01 {
            continue;
        }

        // Apply the force to the acceleration vector.
        acceleration.0 -= transform.translation().normalize() * force;
    }
}

/// A system to perform Verlet integration on the bodies.
fn integrate(time: Res<Time>, mut query: Query<(&mut Acceleration, &mut Transform, &mut LastPos)>) {
    let dt_sq = time.delta_secs() * time.delta_secs();

    // Iterate over each body to update its position.
    for (mut acc, mut transform, mut last_pos) in &mut query {

        // Formula for Verlet integration. Uses two positions instead of velocity and position to calculate
        // the next position of the body. Faster for GPUs to optimize than Euler integration.
        // Damping factor causes energy loss in the system, leading to the spheres coming to rest eventually.
        let new_pos = (2.0 - DAMPING) * transform.translation - (1.0 - DAMPING) * last_pos.0 + acc.0 * dt_sq;

        // Reset acceleration after applying it.
        acc.0 = Vec3::ZERO;
        
        // Update the last position to the current position.
        last_pos.0 = transform.translation;

        // Set the new position of the body.
        transform.translation = new_pos;
    }
}