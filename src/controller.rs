use bevy::prelude::*;

use crate::{Controller, RigidBody};

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement);
    }
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut controller_q: Query<&mut RigidBody, With<Controller>>,
) {
    for mut rigid_body in controller_q.iter_mut() {
        let mut direction = Vec3::default();

        // left
        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }

        // right
        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0
        }

        // up
        if keys.pressed(KeyCode::KeyW) {
            direction.z += 1.0;
        }

        // down
        if keys.pressed(KeyCode::KeyS) {
            direction.z -= 1.0;
        }

        // rotate left
        if keys.pressed(KeyCode::KeyQ) {
            rigid_body.angular_velocity.y += 1.0;
        }

        // rotate right
        if keys.pressed(KeyCode::KeyE) {
            rigid_body.angular_velocity.y -= 1.0;
        }

        // make diagonal movement not any faster
        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        rigid_body.linear_velocity += direction;
    }
}
