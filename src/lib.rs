mod collider_lines;
mod controller;
mod physics;

use bevy::prelude::*;
use collider_lines::ColliderLinesPlugin;
use controller::ControllerPlugin;
use physics::PhysicsPlugin;

pub struct Bevy3dPhysicsLitePlugin;

impl Plugin for Bevy3dPhysicsLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PhysicsPlugin, ColliderLinesPlugin, ControllerPlugin));
    }
}

#[derive(Component)]
pub struct Controller;

#[derive(Component)]
pub struct MapBase;

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
}

#[derive(Component, Default)]
pub struct RigidBody {
    pub damping: Damping,
    pub fixed: bool,
    pub gravity: Gravity,
    pub speed: Speed,
    // pub linear_velocity: Velocity,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub is_colliding: bool,
}

impl RigidBody {
    fn apply_linear_velocity(&mut self, position: &mut Vec3, delta_time: f32) {
        *position += self.linear_velocity * self.speed.0 * delta_time;
    }

    fn apply_damping(&mut self) {
        self.linear_velocity *= 1.0 - self.damping.0;
        self.angular_velocity *= 1.0 - self.damping.0;
    }

    fn apply_gravity(&mut self, position: &mut Vec3, delta_time: f32) {
        if !self.fixed {
            position.y -= self.gravity.0 * delta_time;
        }
    }

    fn apply_angular_velocity(&mut self, rotation: &mut Quat, delta_time: f32) {
        let angular_change = Quat::from_rotation_y(self.angular_velocity.y * delta_time); // Example for Y-axis rotation
        *rotation *= angular_change;
    }
}

#[derive(Component)]
pub struct ColliderLines;
