use bevy::{
    input::mouse::*,
    prelude::*,
};

#[derive(Default)]
pub struct MouseEventState {
    pub mouse_motion_event_reader: EventReader<MouseMotion>,
    pub mouse_wheel_event_reader: EventReader<MouseWheel>,
}


#[derive(Default)]
pub struct InputBindingPlugin;

impl Plugin for InputBindingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<InputBindings>()
            .add_system(update_input.system());
    }
}

#[derive(Debug, Default)]
pub struct InputBindings {
    movement: Vec2,
    look: Vec2,
    do_jump: bool,
    scroll: f32,
}

impl InputBindings {
    pub fn movement(&self) -> Vec2 { self.movement }
    pub fn look(&self) -> Vec2 { self.look }
    pub fn do_jump(&self) -> bool { self.do_jump }
    pub fn scroll(&self) -> f32 { self.scroll }
}


fn update_input(
    mut input: ResMut<InputBindings>,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<MouseEventState>,
    motion_events: Res<Events<MouseMotion>>,
    wheel_events: Res<Events<MouseWheel>>,
) {
    input.movement = Vec2::zero();

    if keyboard_input.pressed(KeyCode::W) {
        input.movement.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::S) {
        input.movement.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::D) {
        input.movement.x += 1.;
    }
    if keyboard_input.pressed(KeyCode::A) {
        input.movement.x -= 1.;
    }

    if input.movement.length_squared() != 0.0 {
        input.movement = input.movement.normalize();
    }

    input.do_jump = keyboard_input.just_pressed(KeyCode::Space);

    input.look = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&motion_events) {
        input.look += event.delta;
    }
    input.look.y = -input.look.y;

    input.scroll = 0.;
    for event in state.mouse_wheel_event_reader.iter(&wheel_events) {
        input.scroll -= event.y;
    }
}