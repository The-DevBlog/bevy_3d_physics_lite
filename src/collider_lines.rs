use bevy::prelude::*;

use crate::Collider;

pub struct ColliderLinesPlugin;

impl Plugin for ColliderLinesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, lines);
    }
}

fn lines(query: Query<(&Collider, &Transform), With<Collider>>, mut gizmos: Gizmos) {
    for (collider, transform) in query.iter() {
        let color = if collider.is_colliding {
            Srgba::RED
        } else {
            Srgba::GREEN
        };

        gizmos.cuboid(
            Transform {
                translation: transform.translation,
                scale: collider.cuboid,
                ..default()
            },
            color,
        );
    }
}
