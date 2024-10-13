use bevy::prelude::*;

use crate::*;

pub struct ColliderLinesPlugin;

impl Plugin for ColliderLinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, lines);
    }
}

fn lines(
    query: Query<(&Collider, &RigidBody, &Transform), With<ColliderLines>>,
    mut gizmos: Gizmos,
) {
    for (collider, rigid_body, transform) in query.iter() {
        let color = if rigid_body.is_colliding {
            Srgba::RED
        } else {
            Srgba::GREEN
        };

        gizmos.cuboid(
            Transform {
                translation: transform.translation,
                rotation: transform.rotation,
                scale: collider.cuboid,
                ..default()
            },
            color,
        );
    }
}
