use game_loop::winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};
use shipyard::*;

use crate::camera::Camera;

#[derive(Debug, Unique, Default)]
pub struct InputState {
    pub cursor_in_window: bool,
    pub cursor_captured: bool,
    pub fullscreen: bool,
    pub forward: bool,
    pub backward: bool,
    pub leftward: bool,
    pub rightward: bool,
}

pub fn keyboard_input_sys(event: KeyboardInput, mut input_state: UniqueViewMut<InputState>) {
    let state = event.state == ElementState::Pressed;

    // Variable for virtual key code if input was not a scan code.
    let mut keycode: Option<VirtualKeyCode> = None;

    match event.scancode {
        // W
        17 => input_state.forward = state,
        // S
        31 => input_state.backward = state,
        // A
        30 => input_state.leftward = state,
        // D
        32 => input_state.rightward = state,
        _ => keycode = event.virtual_keycode,
    }

    // Check virtual key codes.
    if !state {
        return;
    }

    if let Some(keycode) = keycode {
        match keycode {
            VirtualKeyCode::Escape => input_state.cursor_captured = false,
            VirtualKeyCode::F11 => input_state.fullscreen = !input_state.fullscreen,
            _ => {}
        }
    }
}

pub fn mouse_input_sys(
    (dx, dy): (f64, f64),
    input_state: UniqueView<InputState>,
    mut camera: UniqueViewMut<Camera>,
) {
    const SENSITIVITY: f32 = 0.05;

    if !input_state.cursor_captured {
        return;
    }

    let mut new_yaw = camera.yaw + dx as f32 * SENSITIVITY;

    if new_yaw > 360.0 {
        new_yaw -= 360.0;
    } else if new_yaw < 0.0 {
        new_yaw += 360.0;
    }

    camera.yaw = new_yaw;

    let mut new_pitch = camera.pitch + dy as f32 * SENSITIVITY;

    if new_pitch > 90.0 {
        new_pitch = 90.0;
    } else if new_pitch < -90.0 {
        new_pitch = -90.0;
    }

    camera.pitch = new_pitch;
}

pub fn mouse_button_sys(button: &MouseButton, mut input_state: UniqueViewMut<InputState>) {
    if !input_state.cursor_in_window {
        return;
    }

    // left button
    if *button == MouseButton::Left {
        input_state.cursor_captured = true;
    }
}

pub fn move_player_sys(input_state: UniqueView<InputState>, mut camera: UniqueViewMut<Camera>) {
    const MOVEMENT_SPEED: f32 = 0.005;

    if !input_state.cursor_captured {
        return;
    }

    let mut movement = glam::Vec3::new(0.0, 0.0, 0.0);

    if input_state.forward {
        movement.z += 1.0;
    }

    if input_state.backward {
        movement.z -= 1.0;
    }

    if input_state.leftward {
        movement.x -= 1.0;
    }

    if input_state.rightward {
        movement.x += 1.0;
    }

    if movement != glam::Vec3::ZERO {
        movement = movement.normalize() * MOVEMENT_SPEED;
        movement = glam::Mat3::from_rotation_y(camera.yaw.to_radians()) * movement;

        camera.eye += movement;
    }
}
