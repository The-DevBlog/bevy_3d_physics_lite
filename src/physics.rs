use crate::*;
use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_physics, collisions).chain()); // applying physics before collision resolution is more accurate
    }
}

// Check if two AABBs (Axis-Aligned Bounding Boxes) are colliding
fn check_aabb_collision(cuboid_a: &Vec3, cuboid_b: &Vec3, pos_a: &Vec3, pos_b: &Vec3) -> bool {
    let cuboid_a = *cuboid_a / 2.0;
    let cuboid_b = *cuboid_b / 2.0;

    (pos_a.x - pos_b.x).abs() <= (cuboid_a.x + cuboid_b.x)
        && (pos_a.y - pos_b.y).abs() <= (cuboid_a.y + cuboid_b.y)
        && (pos_a.z - pos_b.z).abs() <= (cuboid_a.z + cuboid_b.z)
}

// Resolve the collision between two entities by adjusting their positions and velocities
fn resolve_aabb_collision(
    collider_a: &mut Collider,
    collider_b: &mut Collider,
    pos_a: &mut Vec3,
    pos_b: &mut Vec3,
) {
    let diff = *pos_a - *pos_b;

    // Calculate overlap along each axis
    let cuboid_a = collider_a.cuboid / 2.0;
    let cuboid_b = collider_b.cuboid / 2.0;
    let overlap_x = (cuboid_a.x + cuboid_b.x) - diff.x.abs();
    let overlap_y = (cuboid_a.y + cuboid_b.y) - diff.y.abs();
    let overlap_z = (cuboid_a.z + cuboid_b.z) - diff.z.abs();

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
    if !collider_a.fixed {
        *pos_a += axis * min_overlap * 0.5;
    }

    if !collider_b.fixed {
        *pos_b -= axis * min_overlap * 0.5;
    }

    // Reflect velocities (simple elastic collision)
    let relative_velocity = collider_a.velocity.0 - collider_b.velocity.0;
    let velocity_along_axis = relative_velocity.dot(axis);

    let restitution = 0.8; // Coefficient of restitution (bounciness)
    let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;

    if !collider_a.fixed {
        collider_a.velocity.0 += impulse * axis;
    }

    if !collider_b.fixed {
        collider_b.velocity.0 -= impulse * axis;
    }
}

// main update system for applying
fn collisions(mut query: Query<(&mut Collider, &mut Transform, Option<&MapBase>), With<Collider>>) {
    for (mut collider, _, _) in query.iter_mut() {
        collider.is_colliding = false;
    }

    let mut entities = query.iter_combinations_mut();
    while let Some(
        [(mut collider_a, mut transform_a, mapbase_a), (mut collider_b, mut transform_b, mapbase_b)],
    ) = entities.fetch_next()
    {
        // Check and resolve collisions if they collide
        if check_aabb_collision(
            &collider_a.cuboid,
            &collider_b.cuboid,
            &transform_a.translation,
            &transform_b.translation,
        ) {
            if mapbase_a.is_none() && mapbase_b.is_none() {
                collider_a.is_colliding = true;
                collider_b.is_colliding = true;
            }

            resolve_aabb_collision(
                &mut collider_a,
                &mut collider_b,
                &mut transform_a.translation,
                &mut transform_b.translation,
            );
        }
    }
}

// main system for applying physics
fn apply_physics(mut query: Query<(&mut Collider, &mut Transform)>, time: Res<Time>) {
    for (mut collider, mut transform) in query.iter_mut() {
        collider.apply_damping(); // apply damping BEFORE velocity
        collider.apply_velocity(&mut transform.translation, time.delta_seconds());
        collider.apply_gravity(&mut transform.translation, time.delta_seconds());
    }
}
