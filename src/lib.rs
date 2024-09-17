mod collider_lines;
mod physics;

use bevy::prelude::*;
use collider_lines::ColliderLinesPlugin;
use physics::PhysicsPlugin;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PhysicsPlugin, ColliderLinesPlugin));
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

#[derive(Component)]
pub struct ColliderColor(pub Srgba);

impl Default for ColliderColor {
    fn default() -> Self {
        Self(Srgba::NONE)
    }
}

#[derive(Component, Default)]
pub struct Collider {
    pub cuboid: Vec3,
    pub color: Srgba,
    pub damping: Damping,
    pub fixed: bool,
    pub gravity: Gravity,
    pub speed: Speed,
    pub velocity: Velocity,
    pub is_colliding: bool,
}

impl Collider {
    fn apply_velocity(&mut self, position: &mut Vec3, delta_time: f32) {
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
