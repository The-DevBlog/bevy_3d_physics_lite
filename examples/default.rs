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
        .add_systems(Update, movement)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera = (
        Camera3dBundle {
            transform: Transform::from_xyz(5.0, 3.5, 11.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Name::new("Camera"),
    );

    let light = (
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                ..default()
            },
            transform: Transform {
                rotation: Quat::from_rotation_x(-0.4),
                ..default()
            },
            ..default()
        },
        Name::new("Light"),
    );

    let ground = (
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(15.0, 15.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Collider {
            cuboid: Vec3::new(15.0, 0.1, 15.0),
            // fixed: true,
            // ..default()
        },
        // Collider::cuboid(15.0, 0.1, 15.0),
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

fn movement(keys: Res<ButtonInput<KeyCode>>, mut player_q: Query<(&mut RigidBody), With<Player>>) {
    for mut rigid_body in player_q.iter_mut() {
        let mut direction = Vec3::default();

        if keys.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }

        if keys.pressed(KeyCode::KeyD) {
            direction.x += 1.0
        }

        if keys.pressed(KeyCode::KeyW) {
            direction.z += 1.0;
        }

        if keys.pressed(KeyCode::KeyS) {
            direction.z -= 1.0;
        }

        // make diagonal movement not any faster
        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        rigid_body.velocity.0 += direction;
    }
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::BLACK,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Collider {
            cuboid: Vec3::new(1.0, 1.0, 1.0),
        },
        RigidBody {
            damping: Damping(0.2),
            gravity: Gravity(1.0),
            speed: Speed(2.5),
            ..default()
        },
        ColliderLines,
        Player,
        Name::new("Player"),
    ));
}

fn spawn_object(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 5.0),
            ..default()
        },
        Collider {
            cuboid: Vec3::new(1.0, 1.0, 1.0),
        },
        RigidBody::default(),
        ColliderLines,
        Name::new("Object"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(2.0, 1.0, 5.0),
            ..default()
        },
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
