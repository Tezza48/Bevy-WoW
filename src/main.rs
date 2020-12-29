use bevy::{input::mouse::{MouseMotion, MouseWheel}, prelude::*, render::mesh::{Indices, VertexAttributeValues}};
use bevy_rapier3d::rapier::dynamics::RigidBodyBuilder;
use bevy_rapier3d::rapier::geometry::ColliderBuilder;
use bevy_rapier3d::{
    na::{Point3, Vector3},
    physics::{RapierPhysicsPlugin, RigidBodyHandleComponent},
    rapier::dynamics::{RigidBody, RigidBodyHandle, RigidBodySet},
    render::RapierRenderPlugin,
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
struct MouseState {
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

struct HasLoadedCollider(bool);

struct Terrain;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .init_resource::<MouseState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup.system())
        .add_system(process_mouse_events.system())
        .add_system(update_player.system())
        .add_resource(HasLoadedCollider(false))
        .add_system(load_collider.system())
        .run();
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let cube_mat_handle = materials.add({
        let mut cube_material: StandardMaterial = Color::rgb(1.0, 1.0, 1.0).into();
        cube_material.shaded = true;
        cube_material
    });

    // Spawn camera and player, set entity for camera on player.
    let camera_entity = commands.spawn(Camera3dBundle::default()).current_entity();

    let player_rigid = RigidBodyBuilder::new_dynamic()
        .mass(1., true)
        .translation(0., 30., 0.)
        .linvel(0.0, 50.0, 0.0);
    let player_coll = ColliderBuilder::cuboid(1., 1., 1.);

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
        .with((player_rigid, player_coll))
        .current_entity();

    let terrain_scene = asset_server.load("pentagon.gltf");
    meshes.get(terrain_scene.clone());

    // Debug ground colliders.
    let ground_rigid = RigidBodyBuilder::new_static();
    let ground_collider = ColliderBuilder::cuboid(20.0, 1.0, 20.0);

    commands
        // Append camera to player as child.
        .push_children(player_entity.unwrap(), &[camera_entity.unwrap()])

        // Create the environment.
        .spawn_scene(terrain_scene)
        .with(Terrain)
        // .with((ground_rigid, ground_collider))
        // .with(ground_collider.unwrap())
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        });
}

// System polls whether pentagon mesh is loaded, then initialises physics components for it and the player.
fn load_collider(
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut has_loaded: ResMut<HasLoadedCollider>,
    mut queries: QuerySet<(Query<(Entity, &Terrain)>, Query<(Entity, &MMOPlayer)>)>,
    commands: &mut Commands
) {
    if has_loaded.0 {
        return;
    }

    let terrain_mesh: Handle<Mesh> = asset_server.load("pentagon.gltf#Mesh0/Primitive0");

    match asset_server.get_load_state(terrain_mesh.id) {
        bevy::asset::LoadState::Failed => {
            println!("Failed to Load!");
        },
        bevy::asset::LoadState::Loading => {
            println!("Loading!");
        },
        bevy::asset::LoadState::NotLoaded => {
            println!("Not Loaded");
        },
        bevy::asset::LoadState::Loaded => {
            println!("Loaded");
            if let Some(mesh) = meshes.get(&terrain_mesh) {
                let verts = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();

                let verts: &Vec<[f32; 3]> = match verts {
                    VertexAttributeValues::Float3(vert) => Some(vert),
                    _ => None
                }.unwrap();
                let verts: Vec<Point3<f32>> = verts.iter().map(|vert| {
                    Point3::new(vert[0], vert[1], vert[2])
                }).collect();

                let indices: Vec<Point3<u32>> = match mesh.indices().unwrap() {
                    Indices::U32(i) => Some(i),
                    _ => None,
                }.unwrap().chunks(3).map(|tri| {
                    Point3::new(tri[0], tri[1], tri[2])
                }).collect();

                for (entity, _) in queries.q0().iter() {
                    let ground_rigid = RigidBodyBuilder::new_static();
                    let ground_collider = ColliderBuilder::trimesh(verts.clone(), indices.clone());

                    commands.insert(entity, (ground_rigid, ground_collider));
                }

                for (entity, _) in queries.q1().iter() {
                    let player_rigid = RigidBodyBuilder::new_dynamic()
                        .mass(1.0, true)
                        .translation(0., 30., 0.)
                        .linvel(0.0, 20.0, 0.0);
                    let player_coll = ColliderBuilder::cuboid(1., 1., 1.);


                    commands.insert(entity, (player_rigid, player_coll));
                }
            } else { println!("No mesh was there") }
            has_loaded.0 = true;
        }
    }
}

fn process_mouse_events(
    time: Res<Time>,
    mut state: ResMut<MouseState>,
    motion_events: Res<Events<MouseMotion>>,
    wheel_events: Res<Events<MouseWheel>>,
    mut query: Query<&mut MMOPlayer>,
) {
    let mut look = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&motion_events) {
        look = event.delta;
    }

    let mut zoom_delta = 0.;
    for event in state.mouse_wheel_event_reader.iter(&wheel_events) {
        zoom_delta = event.y;
    }

    let zoom_sense = 10.0;
    let look_sense = 1.0;
    let delta_seconds = time.delta_seconds();

    for mut player in &mut query.iter_mut() {
        player.yaw += look.x * delta_seconds * look_sense;
        player.camera_pitch -= look.y * delta_seconds * look_sense;
        player.camera_distance -= zoom_delta * delta_seconds * zoom_sense;
    }
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut queries: QuerySet<(
        Query<(&mut MMOPlayer, &mut Transform)>,
        Query<&mut Transform>,
    )>,
    // mut rigidbody_Set: ResMut<RigidBodySet>,
) {
    let mut movement = Vec2::zero();

    // let mut jump = keyboard_input.just_pressed(KeyCode::Space);

    if keyboard_input.pressed(KeyCode::W) { movement.y += 1.; }
    if keyboard_input.pressed(KeyCode::S) { movement.y -= 1.; }
    if keyboard_input.pressed(KeyCode::D) { movement.x += 1.; }
    if keyboard_input.pressed(KeyCode::A) { movement.x -= 1.; }

    if movement != Vec2::zero() {
        movement.normalize();
    }

    let move_speed = 10.0;
    movement *= time.delta_seconds() * move_speed;

    let mut cam_positions = Vec::new();

    for (mut player, mut transform) in &mut queries.q0_mut().iter_mut() {
        player.camera_pitch = player
            .camera_pitch
            .max(1f32.to_radians())
            .min(179f32.to_radians());
        player.camera_distance = player.camera_distance.max(5.).min(30.);

        let fwd = transform.forward();
        let right = Vec3::cross(fwd, Vec3::unit_y());

        let fwd = fwd * movement.y;
        let right = right * movement.x;

        transform.translation += Vec3::from(fwd + right);
        transform.rotation = Quat::from_rotation_y(-player.yaw);

        // if jump {
        //     rigidbody_Set.get_mut(rigid.handle())
        //         .unwrap()
        //         .apply_impulse(Vector3::new(0.0, 5.0, 0.0), true);
        // }

        if let Some(camera_entity) = player.camera_entity {
            let cam_pos = Vec3::new(0., player.camera_pitch.cos(), -player.camera_pitch.sin())
                .normalize()
                * player.camera_distance;
            cam_positions.push((camera_entity, cam_pos));
        }
    }

    for (camera_entity, cam_pos) in cam_positions.iter() {
        if let Ok(mut cam_trans) = queries
            .q1_mut()
            .get_component_mut::<Transform>(*camera_entity)
        {
            cam_trans.translation = *cam_pos;

            let look = Mat4::face_toward(
                cam_trans.translation,
                Vec3::zero(),
                Vec3::new(0.0, 1.0, 0.0),
            );
            cam_trans.rotation = look.to_scale_rotation_translation().1;
        }
    }
}
