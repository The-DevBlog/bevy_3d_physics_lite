use bevy::prelude::*;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_physics);
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec3); // 3D velocity

#[derive(Component)]
pub struct Damping(pub f32);

impl Default for Damping {
    fn default() -> Self {
        Damping(0.5)
    }
}

#[derive(Component)]
pub struct Collider {
    pub half_extents: Vec3, // Half the size of the cube along each axis
}

impl Collider {
    pub fn new(half_extents: Vec3) -> Self {
        Self {
            half_extents: Vec3::new(
                half_extents.x / 2.0,
                half_extents.y / 2.0,
                half_extents.z / 2.0,
            ),
        }
    }
}

fn check_aabb_collision(pos_a: Vec3, extents_a: Vec3, pos_b: Vec3, extents_b: Vec3) -> bool {
    (pos_a.x - pos_b.x).abs() <= (extents_a.x + extents_b.x)
        && (pos_a.y - pos_b.y).abs() <= (extents_a.y + extents_b.y)
        && (pos_a.z - pos_b.z).abs() <= (extents_a.z + extents_b.z)
}

fn resolve_aabb_collision(
    pos_a: &mut Vec3,
    vel_a: &mut Vec3,
    extents_a: Vec3,
    pos_b: &mut Vec3,
    vel_b: &mut Vec3,
    extents_b: Vec3,
) {
    let diff = *pos_a - *pos_b;

    // Calculate overlap along each axis
    let overlap_x = (extents_a.x + extents_b.x) - diff.x.abs();
    let overlap_y = (extents_a.y + extents_b.y) - diff.y.abs();
    let overlap_z = (extents_a.z + extents_b.z) - diff.z.abs();

    // Find the axis with the least penetration
    let mut axis = Vec3::ZERO;
    let mut min_overlap = overlap_x;
    axis.x = diff.x.signum();

    if overlap_y < min_overlap {
        min_overlap = overlap_y;
        axis = Vec3::new(0.0, diff.y.signum(), 0.0);
    }

    if overlap_z < min_overlap {
        min_overlap = overlap_z;
        axis = Vec3::new(0.0, 0.0, diff.z.signum());
    }

    // Move objects apart along the axis of least penetration
    *pos_a += axis * min_overlap * 0.5;
    *pos_b -= axis * min_overlap * 0.5;

    // Reflect velocities (simple elastic collision)
    let relative_velocity = *vel_a - *vel_b;
    let velocity_along_axis = relative_velocity.dot(axis);

    // Only resolve if objects are moving toward each other
    if velocity_along_axis < 0.0 {
        let restitution = 0.8; // Coefficient of restitution (bounciness)
        let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;

        *vel_a += impulse * axis;
        *vel_b -= impulse * axis;
    }
}

fn update_physics(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &Damping, &Collider)>,
) {
    let mut entities = query.iter_combinations_mut();

    while let Some(
        [(mut transform_a, mut velocity_a, damping_a, collider_a), (mut transform_b, mut velocity_b, damping_b, collider_b)],
    ) = entities.fetch_next()
    {
        // Apply damping
        velocity_a.0 *= 1.0 - damping_a.0;
        velocity_b.0 *= 1.0 - damping_b.0;

        // Check for AABB collisions
        if check_aabb_collision(
            transform_a.translation,
            collider_a.half_extents,
            transform_b.translation,
            collider_b.half_extents,
        ) {
            // Resolve the collision
            resolve_aabb_collision(
                &mut transform_a.translation,
                &mut velocity_a.0,
                collider_a.half_extents,
                &mut transform_b.translation,
                &mut velocity_b.0,
                collider_b.half_extents,
            );
        }

        // Update positions based on new velocities
        transform_a.translation += velocity_a.0 * time.delta_seconds();
        transform_b.translation += velocity_b.0 * time.delta_seconds();
    }
}
