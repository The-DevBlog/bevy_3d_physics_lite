use bevy::prelude::*;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_physics, collisions).chain()); // applying physics before collision resolution is more accurate
    }
}

#[derive(Component)]
pub struct Gravity(pub f32);

impl Default for Gravity {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Damping(pub f32);

impl Default for Damping {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Component)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Default)]
pub struct Collider {
    pub cuboid: Vec3,
    pub velocity: Velocity,
    pub damping: Damping,
    pub gravity: Gravity,
    pub fixed: bool,
    pub speed: Speed,
}

impl Collider {
    pub fn apply_velocity(&mut self, position: &mut Vec3, delta_time: f32) {
        *position += self.velocity.0 * self.speed.0 * delta_time;
    }

    fn apply_damping(&mut self) {
        self.velocity.0 *= 1.0 - self.damping.0;
    }

    fn apply_gravity(&mut self, position: &mut Vec3, delta_time: f32) {
        if !self.fixed {
            position.y -= self.gravity.0 * delta_time;
        }
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

// main update system for applying collisions
fn collisions(mut query: Query<(&mut Collider, &mut Transform), With<Collider>>) {
    let mut entities = query.iter_combinations_mut();

    while let Some([(mut collider_a, mut transform_a), (mut collider_b, mut transform_b)]) =
        entities.fetch_next()
    {
        // Check and resolve collisions if they collide
        if check_aabb_collision(
            &collider_a.cuboid,
            &collider_b.cuboid,
            &transform_a.translation,
            &transform_b.translation,
        ) {
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
        // apply damping BEFORE velocity
        collider.apply_damping();
        collider.apply_velocity(&mut transform.translation, time.delta_seconds());
        collider.apply_gravity(&mut transform.translation, time.delta_seconds());
    }
}
