use bevy::{input::mouse::*, prelude::*};
use bevy_rapier3d::{na::{Point3, UnitQuaternion, Vector3}, physics::*, rapier::{dynamics::{RigidBodyBuilder, RigidBodySet}, geometry::{ColliderBuilder, ColliderSet, InteractionGroups, Ray}, pipeline::QueryPipeline}};

use crate::{
    interaction_flags,
    resource
};

#[derive(Debug)]
pub struct CharacterController {
    pub yaw: f32,

    pub camera_distance: f32,
    pub camera_pitch: f32,
    pub camera_entity: Option<Entity>,

    pub grounded: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        CharacterController {
            yaw: 0.,

            camera_distance: 20.,
            camera_pitch: 30.0f32.to_radians(),
            camera_entity: None,
            grounded: true,
        }
    }
}

pub fn create_instance(commands: &mut Commands) -> Entity {
    let camera_entity = commands.spawn(Camera3dBundle::default()).current_entity();

    let player_entity = commands
        .spawn((
            CharacterController {
                camera_entity,
                camera_distance: 20.,
                ..Default::default()
            },
            Transform::default(),
            GlobalTransform::default(),
            RigidBodyBuilder::new_dynamic()
                .lock_rotations()
                .mass(1., false)
                .sleeping(true),
            ColliderBuilder::capsule_y(0.25, 0.25)
                .translation(0.0, 0.5, 0.0)
                .collision_groups(InteractionGroups::all().with_groups(interaction_flags::PLAYER))
        )).current_entity().unwrap();

    // Append camera to player as child.
    commands.push_children(player_entity, &[camera_entity.unwrap()]);

    player_entity
}

pub fn process_mouse_events(
    time: Res<Time>,
    mut state: ResMut<resource::MouseState>,
    motion_events: Res<Events<MouseMotion>>,
    wheel_events: Res<Events<MouseWheel>>,
    mut query: Query<&mut CharacterController>,
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

pub fn update_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    query_pipeline: Res<QueryPipeline>,
    colliders: Res<ColliderSet>,
    mut queries: QuerySet<(
        Query<(&mut CharacterController, &Transform, &RigidBodyHandleComponent)>,
        Query<&mut Transform>,
    )>,
    mut rigidbody_set: ResMut<RigidBodySet>,
) {
    let mut movement = Vec2::zero();

    let jump = keyboard_input.just_pressed(KeyCode::Space);

    if keyboard_input.pressed(KeyCode::W) {
        movement.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::S) {
        movement.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::D) {
        movement.x += 1.;
    }
    if keyboard_input.pressed(KeyCode::A) {
        movement.x -= 1.;
    }

    let move_speed = 500.0;
    movement *= time.delta_seconds() * move_speed;

    let mut cam_positions: Vec<(Entity, Vec3)> = Vec::new();

    for (mut player, transform, rigid_handle) in &mut queries.q0_mut().iter_mut() {
        player.camera_pitch = player
            .camera_pitch
            .max(1f32.to_radians())
            .min(90f32.to_radians());
        player.camera_distance = player.camera_distance.max(5.).min(30.);

        let fwd = transform.forward().normalize();
        let right = Vec3::cross(fwd, Vec3::unit_y()).normalize();

        let fwd = fwd * movement.y;
        let right = right * movement.x;

        let direction = Vec3::from(fwd + right);

        let rigid = rigidbody_set.get_mut(rigid_handle.handle()).unwrap();

        let origin = Point3::new(
            transform.translation.x,
            transform.translation.y + 0.3,
            transform.translation.z,
        );
        let ray = Ray::new(origin, Vector3::new(0.0, -1.0, 0.0));

        if let Some((_, _, intersection)) = query_pipeline.cast_ray(
            &colliders,
            &ray,
            0.5,
            InteractionGroups::all().with_mask(!interaction_flags::PLAYER),
        ) {
            let normal = Vector3::new(intersection.normal.x, intersection.normal.y, intersection.normal.z);
            let angle = Vector3::dot(&normal, &Vector3::new(0.0, 1.0, 0.0)).acos().to_degrees();

            // TODO WT: Max slope angle should be on CharacterController.
            player.grounded = angle < 44.0;
        } else {
            player.grounded = false;
        }

        if player.grounded {
            let mut linvel: Vector3<f32> = *rigid.linvel();
            linvel.x = direction.x;
            linvel.z = direction.z;

            rigid.set_linvel(linvel, true);

            if jump {
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
