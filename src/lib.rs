use bevy::prelude::*;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collisions, apply_physics));
    }
}

#[derive(Bundle, Default)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub fixed: Fixed,
    pub gravity: Gravity,
    pub velocity: Velocity,
    pub damping: Damping,
    pub speed: Speed,
}

#[derive(Component)]
pub struct Fixed(pub bool);

impl Default for Fixed {
    fn default() -> Self {
        Self(false)
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
    cuboid: Vec3,
}

impl Collider {
    pub fn cuboid(x: f32, y: f32, z: f32) -> Self {
        Self {
            cuboid: Vec3::new(x / 2.0, y / 2.0, z / 2.0),
        }
    }
}

struct CollisionData {
    position: Vec3,
    velocity: Vec3,
    collider: Vec3,
    fixed: bool,
}

impl CollisionData {
    fn update_transform(&self, transform: &mut Transform) {
        transform.translation = self.position;
    }

    fn update_velocity(&self, velocity: &mut Velocity) {
        velocity.0 = self.velocity;
    }

    fn apply_velocity(&mut self, delta_time: f32) {
        self.position += self.velocity * delta_time;
    }

    fn apply_damping(&mut self, damping: f32) {
        self.velocity *= 1.0 - damping;
    }

    fn apply_gravity(&mut self, gravity: f32, delta_time: f32) {
        self.position.y -= gravity * delta_time;
    }
}

// Check if two AABBs (Axis-Aligned Bounding Boxes) are colliding
fn check_aabb_collision(a: &CollisionData, b: &CollisionData) -> bool {
    (a.position.x - b.position.x).abs() <= (a.collider.x + b.collider.x)
        && (a.position.y - b.position.y).abs() <= (a.collider.y + b.collider.y)
        && (a.position.z - b.position.z).abs() <= (a.collider.z + b.collider.z)
}

// Resolve the collision between two entities by adjusting their positions and velocities
fn resolve_aabb_collision(a: &mut CollisionData, b: &mut CollisionData) {
    let diff = a.position - b.position;

    // Calculate overlap along each axis
    let overlap_x = (a.collider.x + b.collider.x) - diff.x.abs();
    let overlap_y = (a.collider.y + b.collider.y) - diff.y.abs();
    let overlap_z = (a.collider.z + b.collider.z) - diff.z.abs();

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
        a.position += axis * min_overlap * 0.5;
    }

    if !b.fixed {
        b.position -= axis * min_overlap * 0.5;
    }

    // Reflect velocities (simple elastic collision)
    let relative_velocity = a.velocity - b.velocity;
    let velocity_along_axis = relative_velocity.dot(axis);

    let restitution = 0.8; // Coefficient of restitution (bounciness)
    let impulse = -(1.0 + restitution) * velocity_along_axis / 2.0;

    if !a.fixed {
        a.velocity += impulse * axis;
    }

    if !b.fixed {
        b.velocity -= impulse * axis;
    }
}

// The main update system for applying collisions
fn collisions(mut query: Query<(&mut Transform, &mut Velocity, &Collider, &Fixed)>) {
    let mut entities = query.iter_combinations_mut();

    while let Some(
        [(mut transform_a, mut velocity_a, collider_a, fixed_a), (mut transform_b, mut velocity_b, collider_b, fixed_b)],
    ) = entities.fetch_next()
    {
        // Create CollisionData instances for both entities being evaluated
        let mut ent_a = CollisionData {
            position: transform_a.translation,
            velocity: velocity_a.0,
            collider: collider_a.cuboid,
            fixed: fixed_a.0,
        };

        let mut ent_b = CollisionData {
            position: transform_b.translation,
            velocity: velocity_b.0,
            collider: collider_b.cuboid,
            fixed: fixed_b.0,
        };

        // Check and resolve collisions if they collide
        if check_aabb_collision(&ent_a, &ent_b) {
            resolve_aabb_collision(&mut ent_a, &mut ent_b);
        }

        ent_a.update_transform(&mut transform_a);
        ent_b.update_transform(&mut transform_b);

        ent_a.update_velocity(&mut velocity_a);
        ent_b.update_velocity(&mut velocity_b);
    }
}

// main system for applying physics
fn apply_physics(
    mut query: Query<(&Damping, &mut Velocity, &Gravity, &mut Transform, &Speed), With<Collider>>,
    time: Res<Time>,
) {
    for (damping, mut velocity, gravity, mut transform, speed) in query.iter_mut() {
        // add velocity
        transform.translation += velocity.0 * speed.0 * time.delta_seconds();

        // add damping
        velocity.0 *= 1.0 - damping.0;

        // add gravity
        transform.translation.y -= gravity.0 * time.delta_seconds();
    }
}
