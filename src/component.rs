use bevy::{prelude::*, scene::InstanceId};

#[derive(Debug)]
pub struct MMOPlayer {
    pub yaw: f32,

    pub camera_distance: f32,
    pub camera_pitch: f32,
    pub camera_entity: Option<Entity>,

    pub grounded: bool,
}

impl Default for MMOPlayer {
    fn default() -> Self {
        MMOPlayer {
            yaw: 0.,

            camera_distance: 20.,
            camera_pitch: 30.0f32.to_radians(),
            camera_entity: None,
            grounded: true,
        }
    }
}