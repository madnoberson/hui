use getset::Getters;
use winit::event::{ElementState, MouseButton, WindowEvent};

#[derive(Clone, Copy)]
pub enum MouseButtonState {
    Up,
    Down,
}

#[derive(Getters)]
#[get = "pub"]
pub struct InputState {
    mouse_position:     Option<[f32; 2]>,
    left_mouse_button:  MouseButtonState,
    right_mouse_button: MouseButtonState,
}

impl InputState {
    pub fn sync(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_input(state, button)
            }
            _ => {}
        }
    }
}

impl InputState {
    fn on_mouse_input(&mut self, state: &ElementState, button: &MouseButton) {
        let button_state = match button {
            MouseButton::Left => &mut self.left_mouse_button,
            MouseButton::Right => &mut self.right_mouse_button,
            _ => return,
        };
        *button_state = match state {
            ElementState::Pressed => MouseButtonState::Down,
            ElementState::Released => MouseButtonState::Up,
        };
    }
}
