use bevy::prelude::*;
use bevy::{math::bounding::Aabb2d, window::PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls, RtsCameraPlugin};

const GREEN: Hsla = Hsla::hsl(120.0, 0.22, 0.3);
const MAP_SIZE: f32 = 400.0;
const UNITS: i32 = 100;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RtsCameraPlugin,
            RapierDebugRenderPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            WorldInspectorPlugin::new(),
        ))
        .init_resource::<MouseCoords>()
        .init_resource::<BoxCoords>()
        .init_resource::<GameCommands>()
        .add_systems(Startup, (spawn_camera, spawn_map, spawn_unit))
        .add_systems(
            Update,
            (
                set_mouse_coords,
                set_box_coords,
                set_drag_select,
                drag_select,
                single_select,
                set_selected,
                deselect_all,
            )
                .chain()
                .after(set_unit_destination),
        )
        .add_systems(Update, (move_unit, set_unit_destination))
        .run();
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Unit;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct MapBase;

#[derive(Component)]
pub struct Commandable;

#[derive(Component)]
pub struct Destination(pub Option<Vec3>);

#[derive(Resource, Default, Debug)]
pub struct MouseCoords {
    pub global: Vec3,
    pub local: Vec2,
}

#[derive(Resource, Default, Debug)]
pub struct BoxCoords {
    pub global_start: Vec3,
    pub global_end: Vec3,
    pub local_start: Vec2,
    pub local_end: Vec2,
}

impl BoxCoords {
    pub fn empty_global(&mut self) {
        self.global_start = Vec3::ZERO;
        self.global_end = Vec3::ZERO;
    }
}

#[derive(Resource, Default)]
pub struct GameCommands {
    pub drag_select: bool,
    pub single_select: bool,
    pub selected: bool,
}

#[derive(Bundle)]
pub struct UnitBundle {
    pub collider: Collider,
    pub damping: Damping,
    pub external_impulse: ExternalImpulse,
    pub rigid_body: RigidBody,
    pub speed: Speed,
    pub destination: Destination,
    pub unit: Unit,
    pub locked_axis: LockedAxes,
}

impl UnitBundle {
    pub fn new(speed: f32, size: Vec3) -> Self {
        Self {
            collider: Collider::cuboid(size.x, size.y, size.z),
            damping: Damping {
                linear_damping: 5.0,
                ..default()
            },
            external_impulse: ExternalImpulse::default(),
            rigid_body: RigidBody::Dynamic,
            speed: Speed(speed),
            destination: Destination(None),
            unit: Unit,
            locked_axis: (LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z),
        }
    }
}

fn spawn_unit(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut unit =
        |size: f32, speed: f32, translation: Vec3| -> (PbrBundle, Commandable, UnitBundle) {
            (
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(size, size, size)),
                    transform: Transform {
                        translation: translation,
                        ..default()
                    },
                    material: materials.add(Color::BLACK),
                    ..default()
                },
                Commandable,
                UnitBundle::new(speed, Vec3::new(size / 2.0, size / 2.0, size / 2.0)),
            )
        };

    let offset_increment = 2.0;
    for row_index in 0..(UNITS / 10) {
        let offset = row_index as f32 * offset_increment;

        for i in (0..(UNITS / 5)).filter(|&i| i % 2 == 0) {
            cmds.spawn(unit(1.0, 60.0, Vec3::new(i as f32, 0.5, offset)));
        }
    }
}

fn spawn_map(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    cmds.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(MAP_SIZE, MAP_SIZE)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Collider::cuboid(MAP_SIZE / 2.0, 0.0, MAP_SIZE / 2.0),
        Ground,
        MapBase,
        Name::new("Map Base"),
    ));

    // Light
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            150.0f32.to_radians(),
            -40.0f32.to_radians(),
            0.0,
        )),
        ..default()
    });
}

fn spawn_camera(mut cmds: Commands) {
    cmds.spawn((
        Camera3dBundle::default(),
        RtsCamera {
            bounds: Aabb2d::new(Vec2::ZERO, Vec2::new(MAP_SIZE / 2.0, MAP_SIZE / 2.0)),
            min_angle: 60.0f32.to_radians(),
            height_max: 100.0,
            ..default()
        },
        RtsCameraControls {
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyW,
            key_down: KeyCode::KeyS,
            pan_speed: 150.0,
            zoom_sensitivity: 0.2,
            ..default()
        },
    ));
}

fn set_drag_select(box_coords: Res<BoxCoords>, mut game_cmds: ResMut<GameCommands>) {
    let drag_threshold = 2.5;
    let width_z = (box_coords.global_start.z - box_coords.global_end.z).abs();
    let width_x = (box_coords.global_start.x - box_coords.global_end.x).abs();

    if width_z > drag_threshold || width_x > drag_threshold {
        game_cmds.drag_select = true;
    } else {
        game_cmds.drag_select = false;
    }
}

fn set_box_coords(
    mut box_coords: ResMut<BoxCoords>,
    input: Res<ButtonInput<MouseButton>>,
    mouse_coords: Res<MouseCoords>,
) {
    if input.just_pressed(MouseButton::Left) {
        box_coords.global_start = mouse_coords.global;
        box_coords.local_start = mouse_coords.local;
    }

    if input.pressed(MouseButton::Left) {
        box_coords.local_end = mouse_coords.local;
        box_coords.global_end = mouse_coords.global;
    }

    if input.just_released(MouseButton::Left) {
        box_coords.empty_global();
    }
}

// referenced https://bevy-cheatbook.github.io/cookbook/cursor2world.html
fn set_mouse_coords(
    mut mouse_coords: ResMut<MouseCoords>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    cam_q: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    map_base_q: Query<&GlobalTransform, With<MapBase>>,
) {
    let (cam, cam_trans) = cam_q.single();
    let map_base_trans = map_base_q.single();
    let window = window_q.single();
    let Some(local_cursor) = window.cursor_position() else {
        return;
    };

    let plane_origin = map_base_trans.translation();
    let plane = InfinitePlane3d::new(map_base_trans.up());
    // let plane = Plane3d::new(map_base_trans.up());
    let Some(ray) = cam.viewport_to_world(cam_trans, local_cursor) else {
        return;
    };
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        return;
    };
    let global_cursor = ray.get_point(distance);

    mouse_coords.global = global_cursor;
    mouse_coords.local = local_cursor;
}

pub fn drag_select(
    mut cmds: Commands,
    mut gizmos: Gizmos,
    unit_q: Query<(Entity, &Transform), With<Commandable>>,
    // unit_q: Query<(Entity, &Transform), With<Unit>>,
    box_coords: Res<BoxCoords>,
    game_cmds: Res<GameCommands>,
) {
    if !game_cmds.drag_select {
        return;
    }

    let start = box_coords.global_start;
    let end = box_coords.global_end;

    // draw rectangle
    let gray = Color::srgb(0.68, 0.68, 0.68);
    gizmos.line(start, Vec3::new(end.x, 0.0, start.z), gray);
    gizmos.line(start, Vec3::new(start.x, 0.0, end.z), gray);
    gizmos.line(Vec3::new(start.x, 0.0, end.z), end, gray);
    gizmos.line(Vec3::new(end.x, 0.0, start.z), end, gray);

    let min_x = start.x.min(end.x);
    let max_x = start.x.max(end.x);
    let min_z = start.z.min(end.z);
    let max_z = start.z.max(end.z);

    for (unit_ent, unit_trans) in unit_q.iter() {
        // check to see if units are within selection rectangle
        let unit_pos = unit_trans.translation;
        let in_box_bounds = unit_pos.x >= min_x
            && unit_pos.x <= max_x
            && unit_pos.z >= min_z
            && unit_pos.z <= max_z;

        if in_box_bounds {
            cmds.entity(unit_ent).insert((
                // ColliderDebugColor(Color::hsla(120.0, 0.22, 0.3, 0.0)),
                ColliderDebugColor(GREEN),
                Selected,
            ));
        } else {
            cmds.entity(unit_ent)
                .remove::<Selected>()
                .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 1.0)));
        }
    }
}

pub fn single_select(
    mut cmds: Commands,
    rapier_context: Res<RapierContext>,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    select_q: Query<(Entity, &Selected)>,
    mouse_coords: Res<MouseCoords>,
    input: Res<ButtonInput<MouseButton>>,
    game_cmds: Res<GameCommands>,
    commandable_q: Query<&Commandable>,
) {
    if !input.just_released(MouseButton::Left) || game_cmds.drag_select {
        return;
    }

    let (cam, cam_trans) = cam_q.single();

    let Some(ray) = cam.viewport_to_world(cam_trans, mouse_coords.local) else {
        return;
    };

    let hit = rapier_context.cast_ray(
        ray.origin,
        ray.direction.into(),
        f32::MAX,
        true,
        QueryFilter::only_dynamic(),
    );

    if let Some((ent, _)) = hit {
        if commandable_q.get(ent).is_ok() {
            // deselect all currently selected entities
            for (selected_entity, _) in select_q.iter() {
                cmds.entity(selected_entity)
                    .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 1.0)))
                    .remove::<Selected>();
            }

            // select unit
            if !select_q.contains(ent) {
                cmds.entity(ent)
                    .insert((ColliderDebugColor(GREEN), Selected));
            }
        }
    }
}

pub fn deselect_all(
    mut cmds: Commands,
    mut select_q: Query<Entity, With<Selected>>,
    input: Res<ButtonInput<MouseButton>>,
) {
    if input.just_pressed(MouseButton::Right) {
        for ent in select_q.iter_mut() {
            println!("Unit deselected");
            cmds.entity(ent)
                .insert(ColliderDebugColor(Hsla::new(0.0, 0.0, 0.0, 1.0)));
            cmds.entity(ent).remove::<Selected>();
        }
    }
}

fn set_selected(mut game_cmds: ResMut<GameCommands>, select_q: Query<&Selected>) {
    game_cmds.selected = !select_q.is_empty();
}

pub fn set_unit_destination(
    mouse_coords: ResMut<MouseCoords>,
    mut unit_q: Query<(&mut Destination, &Transform), With<Selected>>,
    input: Res<ButtonInput<MouseButton>>,
    game_cmds: Res<GameCommands>,
) {
    if !input.just_released(MouseButton::Left) || game_cmds.drag_select {
        return;
    }

    for (mut unit_destination, trans) in unit_q.iter_mut() {
        let mut destination = mouse_coords.global;
        destination.y += trans.scale.y / 2.0; // calculate for entity height
        unit_destination.0 = Some(destination);
        println!("Unit Moving to ({}, {})", destination.x, destination.y);
    }
}

fn move_unit(
    mut unit_q: Query<
        (
            &mut Transform,
            &mut ExternalImpulse,
            &Speed,
            &mut Destination,
        ),
        With<Unit>,
    >,
    time: Res<Time>,
) {
    for (mut trans, mut ext_impulse, speed, mut destination) in unit_q.iter_mut() {
        if let Some(new_pos) = destination.0 {
            let distance = new_pos - trans.translation;
            if distance.length_squared() <= 5.0 {
                destination.0 = None;
                println!("Unit Stopping");
            } else {
                // Calculate the direction vector on the XZ plane
                let direction = Vec3::new(distance.x, 0.0, distance.z).normalize();

                // Set the impulse to move the unit
                ext_impulse.impulse += direction * speed.0 * time.delta_seconds();

                // Calculate the target Y rotation (yaw)
                let target_yaw = direction.x.atan2(direction.z); // Corrected yaw calculation
                let target_rotation = Quat::from_rotation_y(target_yaw); // Remove adjustment for facing direction

                // Update the unit's rotation to face the direction
                trans.rotation = target_rotation;
            }
        }
    }
}
