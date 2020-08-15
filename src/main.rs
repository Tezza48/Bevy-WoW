use bevy:: {
    prelude::*,
    input::mouse::{ MouseButtonInput, MouseMotion },
};

struct PlayerControl;

struct FaceTowardsParent {
    distance: f32,
    angle: f32,
}

impl Default for FaceTowardsParent {
    fn default() -> Self {
        FaceTowardsParent {
            distance: 10.0,
            angle: 30.0f32.to_radians(),
        }
    }
}

// #[derive(Default)]
// struct InputManager {
//     movement: [f32; 2],
//     look: [f32; 2],
// }

#[derive(Default)]
struct State {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    mouse_motion_event_reader: EventReader<MouseMotion>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

fn main() {
    App::build()
    .add_resource(Msaa { samples: 4 })
    .init_resource::<State>()
    // .init_resource::<InputManager>()
    .add_default_plugins()
    .add_startup_system(setup.system())
    .add_system(mouse_look.system())
    .add_system(move_player.system())
    // .add_system(rotate_player.system())
    .add_system(player_camera_target.system())
    .add_system(camera_control.system())
    .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let cube_mat_handle = materials.add({
        let mut cube_material: StandardMaterial = Color::rgb(1.0, 1.0, 1.0).into();
        cube_material.shaded = true;
        cube_material
    });

    commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat_handle.clone(),
            translation: Translation::new(0.0, 1.0, 0.0),
            ..Default::default()
        })
        .with(PlayerControl)
        .with(Rotation)
        .with_children(|parent| {
            parent
                .spawn(Camera3dComponents {
                    ..Default::default()
                })
                .with(Rotation)
                .with(FaceTowardsParent::default());
        })
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.7, 0.3, 0.0).into()),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, 4.0),
            ..Default::default()
        // })
        // .spawn(Camera3dComponents {
        //     translation: Translation::new(0., 4.0, -20.0),
        //     ..Default::default()
        });
}

// fn process_input_events(mut state: ResMut<State>, mut input: )

fn rotate_player(time: Res<Time>, input: Res<Input<KeyCode>>, mut query: Query<(&PlayerControl, &mut Rotation)>) {
    let mut rot = 0.0;
    if input.pressed(KeyCode::Q) { rot += 1.0 }
    if input.pressed(KeyCode::E) { rot -= 1.0 }

    rot *= time.delta_seconds * 10.0;

    for (_, mut rotation) in &mut query.iter() {

        rotation.0 *= Quat::from_rotation_y(rot);
    }
}

fn mouse_look(
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    mouse_motion_events: Res<Events<MouseMotion>>, 
    mut cams: Query<&mut FaceTowardsParent>,
    mut players: Query<(&PlayerControl, &mut Rotation)>
) {
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        for mut player in &mut cams.iter() {
            player.angle = (player.angle - event.delta.y() * time.delta_seconds * 0.1).max(1f32.to_radians()).min(std::f32::consts::PI - 1f32.to_radians());
            
            println!("angle: {}", player.angle);
        }

        for (_, mut rotation) in &mut players.iter() {

            rotation.0 *= Quat::from_rotation_y(-event.delta.x() * time.delta_seconds * 0.1);
        }
    }

    // for event in state.mouse_button_event_reader.iter(&mouse_button_input_events) {
        
    // }
}

fn camera_control(time: Res<Time>, input: Res<Input<KeyCode>>, mut cams: Query<&mut FaceTowardsParent>) {
    let mut dist = 0.0;
    if input.pressed(KeyCode::R) { dist += time.delta_seconds; }
    if input.pressed(KeyCode::F) { dist -= time.delta_seconds; }

    for mut cam in &mut cams.iter() {
        cam.distance += dist * 5.0;
        cam.distance = cam.distance.max(1.0).min(30.0);
    }
}

fn move_player(time: Res<Time>, input: Res<Input<KeyCode>>, mut query: Query<(&PlayerControl, &Transform, &mut Translation)>) {
    let mut horiz = 0.0;
    let mut verti = 0.0;

    if input.pressed(KeyCode::W) { verti += 1.0 }
    if input.pressed(KeyCode::S) { verti -= 1.0 }
    if input.pressed(KeyCode::D) { horiz += 1.0 }
    if input.pressed(KeyCode::A) { horiz -= 1.0 }

    for (_, transform, mut translation) in &mut query.iter() {
        let fwd = transform.value.z_axis().truncate();
        let right = -transform.value.x_axis().truncate();

        let delta = fwd * verti + right * horiz;
        if delta == Vec3::zero() { continue; }
        let delta = delta.normalize() * time.delta_seconds * 10.0;

        translation.0 += delta;
    }
}

fn player_camera_target(mut look: Query<(&FaceTowardsParent, &mut Translation, &mut Rotation)>) {
    for (face_towards, mut translation, mut rotation) in &mut look.iter() {
        let to = Vec3::new(0., face_towards.angle.cos(), -face_towards.angle.sin()).normalize();
        translation.0 = to * face_towards.distance;

        let look = Mat4::face_toward(translation.0, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
        rotation.0 = look.to_scale_rotation_translation().1;
    }
}