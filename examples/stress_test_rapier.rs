use bevy::{math::bounding::Aabb2d, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_rts_camera::{RtsCamera, RtsCameraControls, RtsCameraPlugin};

const OBJECT_COUNT: usize = 200;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            RtsCameraPlugin,
        ))
        .add_systems(Startup, (setup, spawn_player, spawn_objects))
        .add_systems(Update, movement)
        .run();
}

fn movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_impulse: Query<&mut ExternalImpulse>,
    time: Res<Time>,
) {
    // for mut rigid_body in controller_q.iter_mut() {
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

    // make diagonal movement not any faster
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    if let Ok(mut impulse) = q_impulse.get_single_mut() {
        impulse.impulse = direction * 30.0 * time.delta_secs();
    }
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
        Collider::cuboid(25.0, 0.01, 25.0),
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
        Collider::cuboid(0.5, 0.5, 0.5),
        RigidBody::Dynamic,
        Damping {
            linear_damping: 10.0,
            angular_damping: 20.0,
            ..default()
        },
        ExternalImpulse::default(),
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
    ) {
        (
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
            Transform::from_translation(position),
            Collider::cuboid(0.1, 0.1, 0.1),
            RigidBody::Dynamic,
        )
    };

    for i in 0..OBJECT_COUNT {
        let x = (i as f32 % 10.0) - 0.5;
        let z = (i as f32 / 10.0).floor() - 0.5;
        commands.spawn(obj(Vec3::new(x, 1.0, z)));
    }
}
