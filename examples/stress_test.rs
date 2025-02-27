use bevy::{math::bounding::Aabb2d, prelude::*};
use bevy_3d_physics_lite::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rts_camera::{RtsCamera, RtsCameraControls, RtsCameraPlugin};

const OBJECT_COUNT: usize = 1000;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Bevy3dPhysicsLitePlugin,
            WorldInspectorPlugin::new(),
            RtsCameraPlugin,
        ))
        .add_systems(Startup, (setup, spawn_objects))
        .add_systems(Update, movement)
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = (
        Camera3d::default(),
        RtsCamera {
            bounds: Aabb2d::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
            min_angle: 60.0f32.to_radians(),
            height_max: 50.0,
            ..default()
        },
        RtsCameraControls {
            edge_pan_width: 0.01,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            pan_speed: 75.0,
            zoom_sensitivity: 0.2,
            ..default()
        },
    );

    let light = (
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-0.4)),
        Name::new("Light"),
    );

    let ground = (
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Collider {
            cuboid: Vec3::new(50.0, 0.1, 50.0),
        },
        RigidBody {
            fixed: true,
            ..default()
        },
        MapBase,
        Name::new("Ground"),
    );

    cmds.spawn(camera);
    cmds.spawn(light);
    cmds.spawn(ground);
}

fn movement(
    _keys: Res<ButtonInput<KeyCode>>,
    mut q_rb: Query<(&Transform, &mut RigidBody)>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();
    let impulse_strength = 200.0;

    for (transform, mut rb) in q_rb.iter_mut() {
        let to_center = Vec3::ZERO - transform.translation;

        // println!("To center: {:?}", to_center);
        if to_center.length_squared() > 1e-4 {
            let direction = to_center.normalize();
            rb.linear_velocity = direction * impulse_strength * delta_secs;
        }
        // else {
        //     rb.linear_velocity = Vec3::ZERO;
        // }
    }
}

fn spawn_objects(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut obj = |position: Vec3| -> (
        Mesh3d,
        MeshMaterial3d<StandardMaterial>,
        Transform,
        Collider,
        RigidBody,
        ColliderLines,
        Damping,
    ) {
        (
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
            Transform::from_translation(position),
            Collider {
                cuboid: Vec3::new(0.2, 0.2, 0.2),
            },
            RigidBody {
                gravity: Gravity(0.0),
                damping: Damping(0.25),
                ..default()
            },
            ColliderLines,
            Damping(0.1),
        )
    };

    let grid_size = (OBJECT_COUNT as f32).sqrt().ceil() as usize;
    let spacing = 1.0;
    let offset = (grid_size as f32 - 1.0) / 2.0;

    for i in 0..OBJECT_COUNT {
        let col = i % grid_size;
        let row = i / grid_size;
        let x = col as f32 * spacing - offset * spacing;
        let z = row as f32 * spacing - offset * spacing;
        commands.spawn(obj(Vec3::new(x, 0.2, z)));
    }
}
