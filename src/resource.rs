use bevy::{input::mouse::*, prelude::*, scene::InstanceId};

#[derive(Default)]
pub struct MouseState {
    pub mouse_motion_event_reader: EventReader<MouseMotion>,
    pub mouse_wheel_event_reader: EventReader<MouseWheel>,
}

#[derive(Default)]
pub struct SceneInstance(pub Option<InstanceId>);