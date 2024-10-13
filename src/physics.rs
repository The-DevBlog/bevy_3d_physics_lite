use crate::*;
use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_physics, collisions).chain()); // applying physics before collision resolution is more accurate
    }
}

// Check if two OBBs (Oriented Bounding Boxes) are colliding
fn check_obb_collision(
    half_extents_a: &Vec3,
    half_extents_b: &Vec3,
    pos_a: &Vec3,
    pos_b: &Vec3,
    rot_a: &Quat,
    rot_b: &Quat,
) -> bool {
    // Compute the orientation axes of OBB A and B
    let axes_a = [*rot_a * Vec3::X, *rot_a * Vec3::Y, *rot_a * Vec3::Z];

    let axes_b = [*rot_b * Vec3::X, *rot_b * Vec3::Y, *rot_b * Vec3::Z];

    // Compute the translation vector between OBBs
    let translation = *pos_b - *pos_a;

    // List of axes to test (15 in total)
    let mut axes = Vec::with_capacity(15);

    // Add face normals of A and B
    axes.extend_from_slice(&axes_a);
    axes.extend_from_slice(&axes_b);

    // Add cross products of edges
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() > 1e-6 {
                axes.push(axis.normalize());
            }
        }
    }

    // Now test each axis
    for axis in axes {
        // Project OBB A onto axis
        let r_a = half_extents_a.x * (axes_a[0].dot(axis)).abs()
            + half_extents_a.y * (axes_a[1].dot(axis)).abs()
            + half_extents_a.z * (axes_a[2].dot(axis)).abs();

        // Project OBB B onto axis
        let r_b = half_extents_b.x * (axes_b[0].dot(axis)).abs()
            + half_extents_b.y * (axes_b[1].dot(axis)).abs()
            + half_extents_b.z * (axes_b[2].dot(axis)).abs();

        // Project the distance between centers onto axis
        let d = translation.dot(axis).abs();

        // If projections do not overlap, there is a separating axis
        if d > r_a + r_b {
            return false;
        }
    }

    // No separating axis found
    true
}

// Resolve the collision between two entities by adjusting their positions and velocities
fn resolve_obb_collision(
    ent_a: (&mut Collider, &mut RigidBody, &mut Transform),
    ent_b: (&mut Collider, &mut RigidBody, &mut Transform),
) {
    let (collider_a, rigid_body_a, transform_a) = ent_a;
    let (collider_b, rigid_body_b, transform_b) = ent_b;

    let half_extents_a = collider_a.cuboid / 2.0;
    let half_extents_b = collider_b.cuboid / 2.0;

    let pos_a = transform_a.translation;
    let pos_b = transform_b.translation;

    let rot_a = transform_a.rotation;
    let rot_b = transform_b.rotation;

    // Compute the orientation axes of OBB A and B
    let axes_a = [rot_a * Vec3::X, rot_a * Vec3::Y, rot_a * Vec3::Z];
    let axes_b = [rot_b * Vec3::X, rot_b * Vec3::Y, rot_b * Vec3::Z];

    // Compute the translation vector between OBBs
    let translation = pos_b - pos_a;

    // List of axes to test (15 in total)
    let mut axes = Vec::with_capacity(15);

    // Add face normals of A and B
    axes.extend_from_slice(&axes_a);
    axes.extend_from_slice(&axes_b);

    // Add cross products of edges
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() > 1e-6 {
                axes.push(axis.normalize());
            }
        }
    }

    // Now test each axis and find the axis with minimum penetration
    let mut min_penetration = f32::MAX;
    let mut collision_axis = Vec3::ZERO;

    for axis in axes {
        // Project OBB A onto axis
        let r_a = half_extents_a.x * (axes_a[0].dot(axis)).abs()
            + half_extents_a.y * (axes_a[1].dot(axis)).abs()
            + half_extents_a.z * (axes_a[2].dot(axis)).abs();

        // Project OBB B onto axis
        let r_b = half_extents_b.x * (axes_b[0].dot(axis)).abs()
            + half_extents_b.y * (axes_b[1].dot(axis)).abs()
            + half_extents_b.z * (axes_b[2].dot(axis)).abs();

        // Project the distance between centers onto axis
        let d = translation.dot(axis);

        let penetration = (r_a + r_b) - d.abs();

        // If projections do not overlap, there is a separating axis (should not happen here)
        if penetration < 0.0 {
            // No collision along this axis
            return;
        } else if penetration < min_penetration {
            min_penetration = penetration;
            collision_axis = axis * d.signum();
        }
    }

    // Move objects apart along the collision axis
    if !rigid_body_a.fixed {
        transform_a.translation -= collision_axis * (min_penetration * 0.5);
    }

    if !rigid_body_b.fixed {
        transform_b.translation += collision_axis * (min_penetration * 0.5);
    }

    // Adjust velocities (simple elastic collision)
    let relative_velocity = rigid_body_a.linear_velocity - rigid_body_b.linear_velocity;
    let velocity_along_axis = relative_velocity.dot(collision_axis);

    let restitution = 0.8; // Coefficient of restitution (bounciness)
    let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;

    if !rigid_body_a.fixed {
        rigid_body_a.linear_velocity += impulse * collision_axis;
    }

    if !rigid_body_b.fixed {
        rigid_body_b.linear_velocity -= impulse * collision_axis;
    }
}

// Main update system for collision detection and resolution
fn collisions(
    mut query: Query<
        (
            &mut Collider,
            &mut RigidBody,
            &mut Transform,
            Option<&MapBase>,
        ),
        With<Collider>,
    >,
) {
    for (_, mut rigid_body, _, _) in query.iter_mut() {
        rigid_body.is_colliding = false;
    }

    let mut entities = query.iter_combinations_mut();
    while let Some(
        [(mut collider_a, mut rb_a, mut transform_a, mapbase_a), (mut collider_b, mut rb_b, mut transform_b, mapbase_b)],
    ) = entities.fetch_next()
    {
        // Check and resolve collisions if they collide
        if check_obb_collision(
            &(collider_a.cuboid / 2.0),
            &(collider_b.cuboid / 2.0),
            &transform_a.translation,
            &transform_b.translation,
            &transform_a.rotation,
            &transform_b.rotation,
        ) {
            if mapbase_a.is_none() && mapbase_b.is_none() {
                rb_a.is_colliding = true;
                rb_b.is_colliding = true;
            }

            resolve_obb_collision(
                (&mut collider_a, &mut rb_a, &mut transform_a),
                (&mut collider_b, &mut rb_b, &mut transform_b),
            );
        }
    }
}

// Main system for applying physics
fn apply_physics(mut query: Query<(&mut RigidBody, &mut Transform)>, time: Res<Time>) {
    for (mut rigid_body, mut transform) in query.iter_mut() {
        let delta = time.delta_seconds();

        rigid_body.apply_damping(); // apply damping BEFORE velocity
        rigid_body.apply_linear_velocity(&mut transform.translation, delta);
        rigid_body.apply_angular_velocity(&mut transform.rotation, delta);
        rigid_body.apply_gravity(&mut transform.translation, delta);
    }
}
