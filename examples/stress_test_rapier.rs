use bevy::{math::bounding::Aabb2d, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_rts_camera::{RtsCamera, RtsCameraControls, RtsCameraPlugin};

const OBJECT_COUNT: usize = 5000;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            RtsCameraPlugin,
        ))
        .add_systems(Startup, (setup, spawn_objects))
        .add_systems(Update, movement)
        .run();
}

fn movement(
    _keys: Res<ButtonInput<KeyCode>>,
    mut q_impulse: Query<(&Transform, &mut ExternalImpulse)>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();
    let impulse_strength = 0.01;

    for (transform, mut ext_impulse) in q_impulse.iter_mut() {
        let to_center = Vec3::ZERO - transform.translation;

        if to_center.length_squared() > 1e-4 {
            let direction = to_center.normalize();
            ext_impulse.impulse = direction * impulse_strength * delta_secs;
        }
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
        Collider::cuboid(25.0, 0.01, 25.0),
        Name::new("Ground"),
    );

    cmds.spawn(camera);
    cmds.spawn(light);
    cmds.spawn(ground);
}

fn spawn_objects(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Closure that creates a new object with the given position.
    let mut obj = |position: Vec3| -> (
        Mesh3d,
        MeshMaterial3d<StandardMaterial>,
        Transform,
        Collider,
        GravityScale,
        RigidBody,
        LockedAxes,
        ExternalImpulse,
    ) {
        (
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
            Transform::from_translation(position),
            Collider::cuboid(0.1, 0.1, 0.1),
            GravityScale(0.0),
            RigidBody::Dynamic,
            LockedAxes::TRANSLATION_LOCKED_Y,
            ExternalImpulse::default(),
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
        commands.spawn(obj(Vec3::new(x, 1.0, z)));
    }
}
