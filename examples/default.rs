use bevy::prelude::*;
use bevy_3d_physics_lite::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Bevy3dPhysicsLitePlugin,
            WorldInspectorPlugin::new(),
        ))
        .add_systems(Startup, (setup, spawn_player, spawn_object))
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = (
        Camera3d::default(),
        Transform::from_xyz(5.0, 3.5, 11.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Camera"),
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
        Mesh3d(meshes.add(Plane3d::default().mesh().size(15.0, 15.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Collider {
            cuboid: Vec3::new(15.0, 0.1, 15.0),
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
        Transform::from_xyz(0.0, 0.5, 0.0),
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

fn spawn_object(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
        Transform::from_xyz(0.0, 0.5, 5.0),
        Collider {
            cuboid: Vec3::new(1.0, 1.0, 1.0),
        },
        RigidBody::default(),
        ColliderLines,
        Name::new("Object"),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
        Transform::from_xyz(2.0, 1.0, 5.0),
        Collider {
            cuboid: Vec3::new(1.0, 1.0, 1.0),
        },
        RigidBody {
            gravity: Gravity(2.0),
            ..default()
        },
        ColliderLines,
        Name::new("Object 2"),
    ));
}
