use bevy:: {
    prelude::*,
    input::mouse::{
        MouseMotion,
        MouseWheel,
    },
};

struct MMOPlayer {
    yaw: f32,

    camera_distance: f32,
    camera_pitch: f32,
    camera_entity: Option<Entity>,
}

impl Default for MMOPlayer {
    fn default() -> Self {
        MMOPlayer {
            yaw: 0.,

            camera_distance: 20.,
            camera_pitch: 30.0f32.to_radians(),
            camera_entity: None,
        }
    }
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

fn main() {
    App::build()
    .add_resource(Msaa { samples: 4 })
    .init_resource::<State>()
    .add_default_plugins()
    .add_startup_system(setup.system())
    .add_system(process_mouse_events.system())
    .add_system(update_player.system())
    .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let cube_mat_handle = materials.add({
        let mut cube_material: StandardMaterial = Color::rgb(1.0, 1.0, 1.0).into();
        cube_material.shaded = true;
        cube_material
    });

    // Spawn camera and player, set entity for camera on player.
    let camera_entity = commands
        .spawn(Camera3dComponents::default())
        .current_entity();

    let player_entity = commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat_handle.clone(),
            translation: Translation::new(0.0, 1.0, 0.0),
            ..Default::default()
        })
        .with(MMOPlayer {
            camera_entity,
            camera_distance: 20.,
            ..Default::default()
        })
        .current_entity();

    commands
        // Append camera to player as child.
        .push_children(player_entity.unwrap(), &[camera_entity.unwrap()])

        // Create the environment.
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.7, 0.3, 0.0).into()),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, 4.0),
            ..Default::default()
        });
}
fn process_mouse_events(
    time: Res<Time>,
    mut state: ResMut<State>, 
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    mut query: Query<&mut MMOPlayer>,
) {
    let mut look = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        look = event.delta;
    }

    let mut zoom_delta = 0.;
    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        zoom_delta = event.y;
    }

    let zoom_sense = 10.0;
    let look_sense = 1.0;

    for mut player in &mut query.iter() {
        player.yaw += look.x() * time.delta_seconds;
        player.camera_pitch -= look.y() * time.delta_seconds * look_sense;
        player.camera_distance -= zoom_delta * time.delta_seconds * zoom_sense;
    }
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut MMOPlayer, &mut Translation, &Transform, &mut Rotation)>,
    camera_query: Query<(&mut Translation, &mut Rotation)>,
) {
    let mut movement = Vec2::zero();
    if keyboard_input.pressed(KeyCode::W) { *movement.y_mut() += 1.; }
    if keyboard_input.pressed(KeyCode::S) { *movement.y_mut() -= 1.; }
    if keyboard_input.pressed(KeyCode::D) { *movement.x_mut() += 1.; }
    if keyboard_input.pressed(KeyCode::A) { *movement.x_mut() -= 1.; }

    if movement != Vec2::zero() { movement.normalize(); }

    let move_speed = 10.0;
    movement *= time.delta_seconds * move_speed;

    for (mut player, mut translation, transform, mut rotation) in &mut player_query.iter() {
        player.camera_pitch = player.camera_pitch.max(1f32.to_radians()).min(179f32.to_radians());
        player.camera_distance = player.camera_distance.max(5.).min(30.);

        let fwd = transform.value.z_axis().truncate() * movement.y();
        let right = -transform.value.x_axis().truncate() * movement.x();

        translation.0 += Vec3::from(fwd + right);
        rotation.0 = Quat::from_rotation_y(-player.yaw);

        if let Some(camera_entity) = player.camera_entity {
            let cam_pos = Vec3::new(0., player.camera_pitch.cos(), -player.camera_pitch.sin()).normalize() * player.camera_distance;
            if let Ok(mut cam_trans) = camera_query.get_mut::<Translation>(camera_entity) {
                cam_trans.0 = cam_pos;
            }

            if let Ok(mut camera_rotation) = camera_query.get_mut::<Rotation>(camera_entity) {
                let look = Mat4::face_toward(cam_pos, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
                camera_rotation.0 = look.to_scale_rotation_translation().1;
            }
        }
    }
}