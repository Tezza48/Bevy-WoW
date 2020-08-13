use bevy::prelude::*;

struct PlayerControl;

struct LookAtPlayer;

fn main() {
    App::build()
    .add_resource(Msaa { samples: 4 })
    .add_default_plugins()
    .add_startup_system(setup.system())
    .add_system(rotate_player.system())
    .add_system(targeted_camera.system())
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
                .spawn(PbrComponents {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
                    material: materials.add(Color::BLUE.into()),
                    translation: Translation::new(0., 2., -2.),
                    ..Default::default()
                })
                .with(Rotation)
                .with(LookAtPlayer);
        })
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.7, 0.3, 0.0).into()),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, 4.0),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            translation: Translation::new(0., 4.0, -20.0),
            ..Default::default()
        })
        .spawn(Camera3dComponents {
            transform: Transform::new_sync_disabled(Mat4::face_toward(
                Vec3::new(-6.0, 8.0, 16.0), 
                Vec3::new(0.0, 0.0, 0.0), 
                Vec3::new(0.0, 1.0, 0.0))),
            ..Default::default()
        });
}

fn rotate_player(time: Res<Time>, input: Res<Input<KeyCode>>, mut query: Query<(&PlayerControl, &mut Rotation)>) {
    let mut rot = 0.0;
    if input.pressed(KeyCode::Q) { rot += 1.0 }
    if input.pressed(KeyCode::E) { rot -= 1.0 }

    rot *= time.delta_seconds;

    for (_, mut rotation) in &mut query.iter() {

        rotation.0 *= Quat::from_rotation_y(rot);
    }
}

fn targeted_camera(mut player: Query<(&PlayerControl, &mut Translation)>, mut look: Query<(&LookAtPlayer, &Translation, &mut Rotation)>) {
    for (_, centre) in &mut player.iter() {
        for (_, eye, mut rotation) in &mut look.iter() {
            let look = Mat4::face_toward(eye.0, centre.0, Vec3::new(0.0, 1.0, 0.0));
            rotation.0 = look.to_scale_rotation_translation().1;
        }
    }
}