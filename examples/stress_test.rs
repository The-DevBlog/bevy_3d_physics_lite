use bevy::{math::bounding::Aabb2d, prelude::*};
use bevy_3d_physics_lite::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rts_camera::{RtsCamera, RtsCameraControls, RtsCameraPlugin};

const OBJECT_COUNT: usize = 200;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Bevy3dPhysicsLitePlugin,
            WorldInspectorPlugin::new(),
            RtsCameraPlugin,
        ))
        .add_systems(Startup, (setup, spawn_player, spawn_objects))
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
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
            key_up: KeyCode::ArrowUp,
            key_down: KeyCode::ArrowDown,
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

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::BLACK))),
        Transform::from_xyz(0.0, 0.5, -20.0),
        Collider {
            cuboid: Vec3::new(1.0, 1.0, 1.0),
        },
        RigidBody {
            damping: Damping(0.2),
            gravity: Gravity(1.0),
            speed: Speed(2.5),
            ..default()
        },
        Controller,
        ColliderLines,
        Name::new("Player"),
    ));
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
                gravity: Gravity(2.0),
                damping: Damping(0.25),
                ..default()
            },
            ColliderLines,
            Damping(0.1),
        )
    };

    for i in 0..OBJECT_COUNT {
        let x = (i as f32 % 10.0) - 2.5;
        let z = (i as f32 / 10.0).floor() - 2.5;
        commands.spawn(obj(Vec3::new(x, 5.0, z)));
    }
}
