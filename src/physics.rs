use bevy::prelude::*;
use std::collections::HashMap;

use crate::{Collider, MapBase, RigidBody}; // for integer grid cell indices
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Here we run the physics integration system and then our collision system.
        app.add_systems(Update, (apply_physics, collisions));
    }
}

// -- Collision Checking and Resolution --

/// Checks whether two oriented bounding boxes (OBBs) are colliding using the Separating Axis Theorem.
fn check_obb_collision(
    half_extents_a: &Vec3,
    half_extents_b: &Vec3,
    pos_a: &Vec3,
    pos_b: &Vec3,
    rot_a: &Quat,
    rot_b: &Quat,
) -> bool {
    let axes_a = [*rot_a * Vec3::X, *rot_a * Vec3::Y, *rot_a * Vec3::Z];
    let axes_b = [*rot_b * Vec3::X, *rot_b * Vec3::Y, *rot_b * Vec3::Z];
    let translation = *pos_b - *pos_a;
    let mut axes = Vec::with_capacity(15);
    axes.extend_from_slice(&axes_a);
    axes.extend_from_slice(&axes_b);
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() > 1e-6 {
                axes.push(axis.normalize());
            }
        }
    }
    for axis in axes {
        let r_a = half_extents_a.x * (axes_a[0].dot(axis)).abs()
            + half_extents_a.y * (axes_a[1].dot(axis)).abs()
            + half_extents_a.z * (axes_a[2].dot(axis)).abs();
        let r_b = half_extents_b.x * (axes_b[0].dot(axis)).abs()
            + half_extents_b.y * (axes_b[1].dot(axis)).abs()
            + half_extents_b.z * (axes_b[2].dot(axis)).abs();
        let d = translation.dot(axis).abs();
        if d > r_a + r_b {
            return false;
        }
    }
    true
}

/// Resolves the collision between two OBBs by moving them apart along the collision axis
/// and adjusting their velocities (a simple elastic collision).
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

    let axes_a = [rot_a * Vec3::X, rot_a * Vec3::Y, rot_a * Vec3::Z];
    let axes_b = [rot_b * Vec3::X, rot_b * Vec3::Y, rot_b * Vec3::Z];
    let translation = pos_b - pos_a;
    let mut axes = Vec::with_capacity(15);
    axes.extend_from_slice(&axes_a);
    axes.extend_from_slice(&axes_b);
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() > 1e-6 {
                axes.push(axis.normalize());
            }
        }
    }

    let mut min_penetration = f32::MAX;
    let mut collision_axis = Vec3::ZERO;
    for axis in axes {
        let r_a = half_extents_a.x * (axes_a[0].dot(axis)).abs()
            + half_extents_a.y * (axes_a[1].dot(axis)).abs()
            + half_extents_a.z * (axes_a[2].dot(axis)).abs();
        let r_b = half_extents_b.x * (axes_b[0].dot(axis)).abs()
            + half_extents_b.y * (axes_b[1].dot(axis)).abs()
            + half_extents_b.z * (axes_b[2].dot(axis)).abs();
        let d = translation.dot(axis);
        let penetration = (r_a + r_b) - d.abs();
        if penetration < 0.0 {
            return; // no collision on this axis (should not happen if already colliding)
        } else if penetration < min_penetration {
            min_penetration = penetration;
            collision_axis = axis * d.signum();
        }
    }

    // Separate the objects
    if !rigid_body_a.fixed {
        transform_a.translation -= collision_axis * (min_penetration * 0.5);
    }
    if !rigid_body_b.fixed {
        transform_b.translation += collision_axis * (min_penetration * 0.5);
    }

    // Simple elastic collision response
    let relative_velocity = rigid_body_a.linear_velocity - rigid_body_b.linear_velocity;
    let velocity_along_axis = relative_velocity.dot(collision_axis);
    let restitution = 0.8;
    let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;
    if !rigid_body_a.fixed {
        rigid_body_a.linear_velocity += impulse * collision_axis;
    }
    if !rigid_body_b.fixed {
        rigid_body_b.linear_velocity -= impulse * collision_axis;
    }
}

// -- Physics Integration System --

fn apply_physics(mut query: Query<(&mut RigidBody, &mut Transform)>, time: Res<Time>) {
    for (mut rigid_body, mut transform) in query.iter_mut() {
        let delta = time.delta_secs();
        rigid_body.apply_damping();
        rigid_body.apply_linear_velocity(&mut transform.translation, delta);
        rigid_body.apply_angular_velocity(&mut transform.rotation, delta);
        rigid_body.apply_gravity(&mut transform.translation, delta);
    }
}

// -- Uniform Grid Collision System --

fn collisions(
    mut query: Query<
        (
            Entity,
            &mut Collider,
            &mut RigidBody,
            &mut Transform,
            Option<&MapBase>,
        ),
        With<Collider>,
    >,
) {
    // Choose a cell size based on your game’s scale.
    const CELL_SIZE: f32 = 1.0;

    // Build the spatial grid.
    // The key is an IVec3 cell index; the value is a vector of tuples:
    // (Entity, position, rotation, half_extents)
    let mut grid: HashMap<IVec3, Vec<(Entity, Vec3, Quat, Vec3)>> = HashMap::new();

    for (entity, collider, mut rb, transform, _mapbase) in query.iter_mut() {
        rb.is_colliding = false;

        let half_extents = collider.cuboid / 2.0;
        let pos = transform.translation;
        let cell = (pos / CELL_SIZE).floor().as_ivec3();
        grid.entry(cell)
            .or_default()
            .push((entity, pos, transform.rotation, half_extents));
    }

    // Helper: return all neighbor cell indices (including the cell itself) for a given cell.
    fn neighbor_cells(cell: IVec3) -> Vec<IVec3> {
        let mut neighbors = Vec::with_capacity(27);
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    neighbors.push(cell + IVec3::new(x, y, z));
                }
            }
        }
        neighbors
    }

    // In order to avoid duplicate collision resolutions, store unique pairs.
    let mut collision_pairs: Vec<(Entity, Entity)> = Vec::new();

    // For each grid cell, check collisions between colliders in that cell and in neighbor cells.
    for (&cell, colliders) in grid.iter() {
        // Collect candidate colliders: those in the current cell plus those in adjacent cells.
        let mut candidates = colliders.clone();
        for neighbor in neighbor_cells(cell) {
            if neighbor == cell {
                continue;
            }
            if let Some(neighbor_colliders) = grid.get(&neighbor) {
                candidates.extend_from_slice(neighbor_colliders);
            }
        }

        // For each collider in the current cell, test against all candidates.
        for i in 0..colliders.len() {
            let (entity_a, pos_a, rot_a, half_extents_a) = colliders[i];
            for candidate in candidates.iter() {
                let (entity_b, pos_b, rot_b, half_extents_b) = *candidate;
                // Avoid self-collision and duplicate pairs by enforcing an order.
                if entity_a >= entity_b {
                    continue;
                }
                if check_obb_collision(
                    &half_extents_a,
                    &half_extents_b,
                    &pos_a,
                    &pos_b,
                    &rot_a,
                    &rot_b,
                ) {
                    collision_pairs.push((entity_a, entity_b));
                }
            }
        }
    }

    // Now, resolve each collision pair.
    for (entity_a, entity_b) in collision_pairs {
        // Use get_many_mut to obtain mutable references to both entities at once.
        if let Ok([mut data_a, mut data_b]) = query.get_many_mut([entity_a, entity_b]) {
            // Destructure the tuple for each entity.
            let (_, ref mut collider_a, ref mut rb_a, ref mut transform_a, _) = data_a;
            let (_, ref mut collider_b, ref mut rb_b, ref mut transform_b, _) = data_b;

            rb_a.is_colliding = true;
            rb_b.is_colliding = true;

            resolve_obb_collision(
                (&mut *collider_a, &mut *rb_a, &mut *transform_a),
                (&mut *collider_b, &mut *rb_b, &mut *transform_b),
            );
        }
    }
}
