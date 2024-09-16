use bevy::prelude::*;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collisions, apply_physics));
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
    pub fn apply_velocity(&mut self, velocity: Vec3) {
        self.velocity.0 += velocity;
    }

    fn apply_damping(&mut self) {
        self.velocity.0 *= 1.0 - self.damping.0;
    }

    fn apply_gravity(&mut self, position: &mut Vec3, delta_time: f32) {
        *position -= self.gravity.0 * delta_time;
    }
}

// Check if two AABBs (Axis-Aligned Bounding Boxes) are colliding
fn check_aabb_collision(a: &Vec3, b: &Vec3) -> bool {
    (a.x - b.x).abs() <= (a.x + b.x)
        && (a.y - b.y).abs() <= (a.y + b.y)
        && (a.z - b.z).abs() <= (a.z + b.z)
}

// Resolve the collision between two entities by adjusting their positions and velocities
fn resolve_aabb_collision(a: &mut Collider, b: &mut Collider, pos_a: &mut Vec3, pos_b: &mut Vec3) {
    let diff = *pos_a - *pos_b;

    // Calculate overlap along each axis
    let overlap_x = (a.cuboid.x + b.cuboid.x) - diff.x.abs();
    let overlap_y = (a.cuboid.y + b.cuboid.y) - diff.y.abs();
    let overlap_z = (a.cuboid.z + b.cuboid.z) - diff.z.abs();

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
    if !a.fixed {
        *pos_a += axis * min_overlap * 0.5;
    }

    if !b.fixed {
        *pos_b -= axis * min_overlap * 0.5;
    }

    // Reflect velocities (simple elastic collision)
    let relative_velocity = a.velocity.0 - b.velocity.0;
    let velocity_along_axis = relative_velocity.dot(axis);

    let restitution = 0.8; // Coefficient of restitution (bounciness)
    let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;

    if !a.fixed {
        a.velocity.0 += impulse * axis;
    }

    if !b.fixed {
        b.velocity.0 -= impulse * axis;
    }
}

// The main update system for applying collisions
fn collisions(mut query: Query<(&mut Collider, &mut Transform), With<Collider>>) {
    let mut entities = query.iter_combinations_mut();

    while let Some([(mut collider_a, mut transform_a), (mut collider_b, mut transform_b)]) =
        entities.fetch_next()
    {
        // Check and resolve collisions if they collide
        if check_aabb_collision(&transform_a.translation, &transform_b.translation) {
            println!("Collision Detected!!!");
            resolve_aabb_collision(
                &mut collider_a,
                &mut collider_b,
                &mut transform_a.translation,
                &mut transform_b.translation,
            );
        }

        // ent_a.update_transform(&mut transform_a);
        // ent_b.update_transform(&mut transform_b);

        // ent_a.update_velocity(&mut velocity_a);
        // ent_b.update_velocity(&mut velocity_b);
    }
}

// main system for applying physics
fn apply_physics(mut query: Query<(&mut Collider, &mut Transform)>, time: Res<Time>) {
    for (mut collider, mut transform) in query.iter_mut() {
        // add velocity
        transform.translation += collider.velocity.0 * collider.speed.0 * time.delta_seconds();

        // add damping
        let damping = collider.damping.0;
        collider.velocity.0 *= 1.0 - damping;

        // add gravity
        transform.translation.y -= collider.gravity.0 * time.delta_seconds();
    }
}
