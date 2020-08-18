use bevy:: {
    prelude::*,
    input::mouse::MouseMotion,
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
struct InputManager {
    movement: Vec2,
    look: Vec2,
    camera_zoom: f32,
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
}

fn main() {
    App::build()
    .add_resource(Msaa { samples: 4 })
    .init_resource::<State>()
    .init_resource::<InputManager>()
    .add_default_plugins()
    .add_startup_system(setup.system())
    .add_system(clear_input_manager.system())
    .add_system(process_mouse_events.system())
    .add_system(process_keys.system())
    .add_system(player_input.system())

    // Actual system for updating the player and camera based on input.
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

fn clear_input_manager(mut input: ResMut<InputManager>) {
    input.look = Vec2::zero(); // This one is event driven so needs to be cleared at the start of the frame.
}

fn process_mouse_events(
    mut state: ResMut<State>, 
    mut input_manager: ResMut<InputManager>, 
    mouse_motion_events: Res<Events<MouseMotion>>
) {
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        input_manager.look = event.delta;
    }
}

fn process_keys(input: Res<Input<KeyCode>>, mut manager: ResMut<InputManager>) {
    manager.movement = Vec2::zero();
    if input.pressed(KeyCode::W) { *manager.movement.y_mut() += 1.; }
    if input.pressed(KeyCode::S) { *manager.movement.y_mut() -= 1.; }
    if input.pressed(KeyCode::D) { *manager.movement.x_mut() += 1.; }
    if input.pressed(KeyCode::A) { *manager.movement.x_mut() -= 1.; }

    if manager.movement != Vec2::zero() { manager.movement.normalize(); }

    manager.camera_zoom = 0.;
    if input.pressed(KeyCode::R) { manager.camera_zoom += 1.; }
    if input.pressed(KeyCode::F) { manager.camera_zoom -= 1.; }
}

fn player_input(
    time: Res<Time>,
    input: Res<InputManager>,
    mut query: Query<&mut MMOPlayer>,
) {
    let rot = input.look * time.delta_seconds;
    let dist = input.camera_zoom * time.delta_seconds;

    for mut player in &mut query.iter() {
        player.yaw += rot.x();
        player.camera_pitch = (player.camera_pitch + rot.y()).max(1f32.to_radians()).min(179f32.to_radians());

        player.camera_distance += dist * 5.0;
        player.camera_distance = player.camera_distance.max(1.0).min(30.0);
    }
}

fn update_player(
    time: Res<Time>, 
    input: Res<InputManager>, 
    mut player_query: Query<(&MMOPlayer, &Transform, &mut Translation, &mut Rotation)>,
    cameras: Query<(&mut Translation, &mut Rotation)>
) {
    for (player, transform, mut translation, mut rotation) in &mut player_query.iter() {
        let fwd = transform.value.z_axis().truncate() * input.movement.y();
        let right = transform.value.x_axis().truncate() * input.movement.x();
    
        // TODO WT: Either normalize the direction or normalize the input.movement outside (latter is better for performance here);
    
        translation.0 += Vec3::from(fwd + right) * time.delta_seconds;

        rotation.0 = Quat::from_rotation_y(-player.yaw);

        if let Some(camera_entity) = player.camera_entity {
            let cam_pos = Vec3::new(0., player.camera_pitch.cos(), -player.camera_pitch.sin()).normalize() * player.camera_distance;
            if let Ok(mut cam_trans) = cameras.get_mut::<Translation>(camera_entity) {
                cam_trans.0 = cam_pos;
            }

            if let Ok(mut camera_rotation) = cameras.get_mut::<Rotation>(camera_entity) {
                let look = Mat4::face_toward(cam_pos, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
                camera_rotation.0 = look.to_scale_rotation_translation().1;
            }
        }
    }
}