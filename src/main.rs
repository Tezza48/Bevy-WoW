mod setup;
mod component;
mod resource;

use bevy::{
    prelude::*,
    input::mouse::{
        MouseMotion,
        MouseWheel
    },
    render::mesh::{
        Indices,
        VertexAttributeValues
    }
};
use bevy_rapier3d::{na::{Isometry3, Point3, UnitQuaternion, Vector3}, physics::*, rapier::dynamics::{
        RigidBodyBuilder,
        RigidBodySet,
    }, rapier::{
        geometry::*,
        pipeline::QueryPipeline
    }};

use resource::*;
use component::*;

struct InteractionFlags;
impl InteractionFlags {
    const PLAYER: u16 = 0b1;
    const WALKABLE: u16 = 0b10;
}

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .init_resource::<resource::MouseState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(setup.system())
        .add_system(process_mouse_events.system())
        .add_system(update_player.system())
        .add_system(load_collider.system())
        // .add_system(physics_events.system())
        .add_resource(SceneInstance::default())
        .run();
}

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

    let terrain_handle = asset_server.load("character_controller_playground.gltf");

    let instance_id = scene_spawner.spawn(terrain_handle.clone());
    scene_instance.0 = Some(instance_id);

    // Spawn camera and player, set entity for camera on player.
    let camera_entity = commands.spawn(Camera3dBundle::default()).current_entity();

    let player_entity = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
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

// System polls whether pentagon mesh is loaded, then initialises physics components for it and the player.
// TODO WT: Run criteria for this system.
fn load_collider(
    meshes: ResMut<Assets<Mesh>>,
    scene_spawner: ResMut<SceneSpawner>,
    scene_instance: Res<SceneInstance>,
    queries: QuerySet<(
        Query<&Handle<Mesh>>,
        Query<(Entity, &MMOPlayer), Without<RigidBodyHandleComponent>>,
    )>,
    commands: &mut Commands,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    if let Some(instance_id) = scene_instance.0 {
        if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
            entity_iter.for_each(|entity| {
                if let Ok(mesh_handle) = queries.q0().get(entity) {
                    if let Some(mesh) = meshes.get(mesh_handle) {
                        let groups = InteractionGroups::all().with_groups(InteractionFlags::WALKABLE);
                        let collider = create_collider_for_mesh(mesh)
                            .collision_groups(groups);
                        
                        let rigid = RigidBodyBuilder::new_static();

                        commands.insert(entity, (collider, rigid));
                    }
                }

                *done = true;
            });
        }
    }

    if *done {
        for (entity, _) in queries.q1().iter() {
            let player_rigid = RigidBodyBuilder::new_dynamic()
                .lock_rotations()
                .linvel(10.0, 10.0, 0.0)
                .mass(1., false)
                .user_data(entity.to_bits() as u128);
            
            let player_collider = ColliderBuilder::capsule_y(0.25, 0.25)
                .collision_groups(InteractionGroups::all().with_groups(InteractionFlags::PLAYER));

            commands.insert(entity, (player_rigid, player_collider));
        }
    }
}

fn create_collider_for_mesh(mesh: &Mesh) -> ColliderBuilder {
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

    ColliderBuilder::trimesh(verts.clone(), indices.clone())
}

fn process_mouse_events(
    time: Res<Time>,
    mut state: ResMut<resource::MouseState>,
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

    let tau = 2. * std::f32::consts::PI;

    for mut player in &mut query.iter_mut() {
        player.yaw += look.x * delta_seconds * look_sense;
        player.yaw = (player.yaw + tau) % tau;
        player.camera_pitch -= look.y * delta_seconds * look_sense;
        player.camera_distance -= zoom_delta * delta_seconds * zoom_sense;
    }
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    query_pipeline: Res<QueryPipeline>,
    colliders: Res<ColliderSet>,
    mut queries: QuerySet<(
        Query<(&mut MMOPlayer, &Transform, &RigidBodyHandleComponent, &ColliderHandleComponent)>,
        Query<&mut Transform>,
    )>,
    mut rigidbody_set: ResMut<RigidBodySet>,
) {
    let mut movement = Vec2::zero();

    let jump = keyboard_input.just_pressed(KeyCode::Space);

    if keyboard_input.pressed(KeyCode::W) { movement.y += 1.; }
    if keyboard_input.pressed(KeyCode::S) { movement.y -= 1.; }
    if keyboard_input.pressed(KeyCode::D) { movement.x += 1.; }
    if keyboard_input.pressed(KeyCode::A) { movement.x -= 1.; }

    let move_speed = 500.0;
    movement *= time.delta_seconds() * move_speed;

    let mut cam_positions: Vec<(Entity, Vec3)> = Vec::new();

    for (mut player, transform, rigid_handle, collider) in &mut queries.q0_mut().iter_mut() {
        player.camera_pitch = player
            .camera_pitch
            .max(1f32.to_radians())
            .min(179f32.to_radians());
        player.camera_distance = player.camera_distance.max(5.).min(30.);

        let fwd = transform.forward().normalize();
        let right = Vec3::cross(fwd, Vec3::unit_y()).normalize();

        let fwd = fwd * movement.y;
        let right = right * movement.x;

        let direction = Vec3::from(fwd + right);
        // if direction.length_squared() > 0 {
        //     direction.normalize();
        // }

        let rigid = rigidbody_set.get_mut(rigid_handle.handle()).unwrap();

        let origin = Point3::new(transform.translation.x, transform.translation.y, transform.translation.z);
        let ray = Ray::new(origin, Vector3::new(0.0, -1.0, 0.0));

        if let Some(interaction) = query_pipeline.cast_ray(&colliders, &ray, 0.7, InteractionGroups::all().with_mask(!InteractionFlags::PLAYER)) {
            player.grounded = true;
        } else {
            player.grounded = false;
        }

        // if !player.grounded { direction /= 2.; }

        if player.grounded {
            let mut linvel: Vector3<f32> = *rigid.linvel();
            linvel.x = direction.x;
            linvel.z = direction.z;
            rigid.set_linvel(linvel, true);

            if  jump {
                rigid.apply_impulse(Vector3::new(0.0, 10.0, 0.0), true);
            }
        }

        let mut position = *rigid.position();
        position.rotation = UnitQuaternion::new(Vector3::y() * -player.yaw);
        rigid.set_position(position, false);

        if let Some(camera_entity) = player.camera_entity {
            let cam_pos = Vec3::new(0., player.camera_pitch.cos(), -player.camera_pitch.sin())
            .normalize()
            * player.camera_distance;
            cam_positions.push((camera_entity, cam_pos));
        }
    }

    for (camera_entity, cam_pos) in cam_positions.iter() {
        if let Ok(mut cam_trans) = queries.q1_mut().get_mut(*camera_entity) {
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
