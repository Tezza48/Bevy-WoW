use bevy::{prelude::*};
use crate::component::*;
use crate::resource::*;

pub fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_instance: ResMut<SceneInstance>,
) {
    let cube_mat_handle = materials.add({
        let mut cube_material: StandardMaterial = Color::rgb(1.0, 1.0, 1.0).into();
        cube_material.shaded = true;
        cube_material
    });

    let terrain_handle = asset_server.load("terrain.gltf");

    commands
        .spawn(())
        .spawn_scene(terrain_handle.clone());

    let instance_id = scene_spawner.spawn(terrain_handle.clone());
    scene_instance.0 = Some(instance_id);

    //Create the environment.
    //commands
        // .spawn(())
        // .spawn_scene(asset_server.load("terrain.gltf"));

    // Spawn camera and player, set entity for camera on player.
    let camera_entity = commands.spawn(Camera3dBundle::default()).current_entity();

    let player_entity = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat_handle.clone(),
            ..Default::default()
        })
        .with(MMOPlayer {
            camera_entity,
            camera_distance: 20.,
            ..Default::default()
        })
        .current_entity();

    // Append camera to player as child.
    commands.push_children(player_entity.unwrap(), &[camera_entity.unwrap()]);

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
        ..Default::default()
    });
}