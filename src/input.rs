use std::collections::HashMap;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, ScanCode};

pub struct Input {
    /// The keys and their states.
    keys: HashMap<ScanCode, InputState>,
    /// The mouse buttons and their states.
    buttons: HashMap<MouseButton, InputState>,
    /// The position of the mouse on the window.
    pub mouse_pos: [f32; 2],
    /// Amount of motion this update.
    pub mouse_mov: [f32; 2],
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashMap::new(),
            buttons: HashMap::new(),
            mouse_pos: [0.0, 0.0],
            mouse_mov: [0.0, 0.0],
        }
    }

    /// Check if a mouse button is pressed.
    pub fn is_button(&self, button: MouseButton, state: InputState) -> bool {
        let actual = *self.buttons.get(&button).unwrap_or(&InputState::Up);
        match state {
            InputState::Down => actual == state || actual == InputState::Pressed,
            InputState::Up => actual == state || actual == InputState::Released,
            _ => actual == state,
        }
    }

    /// Check if a keyboard key is pressed.
    pub fn is_key(&self, key: Key, state: InputState) -> bool {
        let key = key.into();
        let actual = *self.keys.get(&key).unwrap_or(&InputState::Up);
        match state {
            InputState::Down => actual == state || actual == InputState::Pressed,
            InputState::Up => actual == state || actual == InputState::Released,
            _ => actual == state,
        }
    }

    pub(crate) fn update(&mut self) {
        self.keys.iter_mut().for_each(|(_, state)| match state {
            InputState::Pressed => *state = InputState::Down,
            InputState::Released => *state = InputState::Up,
            _ => {}
        });
        self.keys.retain(|_, state| *state != InputState::Released);

        self.buttons.iter_mut().for_each(|(_, state)| match state {
            InputState::Pressed => *state = InputState::Down,
            InputState::Released => *state = InputState::Up,
            _ => {}
        });
        self.buttons
            .retain(|_, state| *state != InputState::Released);

        self.mouse_mov = [0.0, 0.0];
    }

    pub(crate) fn handle_key(&mut self, input: KeyboardInput) {
        match input.state {
            ElementState::Pressed => {
                if self.keys.get(&input.scancode) != Some(&InputState::Down) {
                    self.keys.insert(input.scancode, InputState::Pressed);
                }
            }
            ElementState::Released => {
                self.keys.insert(input.scancode, InputState::Released);
            }
        }
    }

    pub(crate) fn handle_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.buttons.get(&button) != Some(&InputState::Down) {
                    self.buttons.insert(button, InputState::Pressed);
                }
            }
            ElementState::Released => {
                self.buttons.insert(button, InputState::Released);
            }
        }
    }

    pub(crate) fn handle_cursor(&mut self, pos: PhysicalPosition<f64>) {
        let old_pos = self.mouse_pos;
        self.mouse_pos = [pos.x as f32, pos.y as f32];
        self.mouse_mov = [
            self.mouse_pos[0] - old_pos[0],
            self.mouse_pos[1] - old_pos[1],
        ];
    }
}

/// The state of any of the inputs. The difference between pressed
/// and down is that down fires while it's held down, and pressed
/// is only for the first frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputState {
    Up,
    Pressed,
    Down,
    Released,
}

/// An abstraction of the keyboard scancode that automatically
/// applies itself to the correct system's keyboard to scancode
/// layout.
#[derive(Clone, Copy, Debug)]
pub enum Key {
    Escape,
    Num(u8),
    Letter(char),
    Other(ScanCode),
}

impl From<Key> for ScanCode {
    #[cfg(target_os = "linux")]
    fn from(key: Key) -> Self {
        match key {
            Key::Escape => 1,
            Key::Num(0) => 11,
            Key::Num(num) => {
                if num < 10 {
                    num as u32 + 1
                } else {
                    panic!("Invalid key number {num}");
                }
            }
            Key::Letter('q') => 16,
            Key::Letter('w') => 17,
            Key::Letter('e') => 18,
            Key::Letter('r') => 19,
            Key::Letter('t') => 20,
            Key::Letter('y') => 21,
            Key::Letter('u') => 22,
            Key::Letter('i') => 23,
            Key::Letter('o') => 24,
            Key::Letter('p') => 25,
            Key::Letter('a') => 30,
            Key::Letter('s') => 31,
            Key::Letter('d') => 32,
            Key::Letter('f') => 33,
            Key::Letter('g') => 34,
            Key::Letter('h') => 35,
            Key::Letter('j') => 36,
            Key::Letter('k') => 37,
            Key::Letter('l') => 38,
            Key::Letter('z') => 44,
            Key::Letter('x') => 45,
            Key::Letter('c') => 46,
            Key::Letter('v') => 47,
            Key::Letter('b') => 48,
            Key::Letter('n') => 49,
            Key::Letter('m') => 50,
            Key::Letter(key) => panic!("Invalid key {key}"),
            Key::Other(scancode) => scancode,
        }
    }

    #[cfg(target_os = "macos")]
    fn from(key: Key) -> Self {
        match key {
            Key::Letter('a') => 0,
            Key::Letter('s') => 1,
            Key::Letter('d') => 2,
            Key::Letter('f') => 3,
            Key::Letter('h') => 4,
            Key::Letter('g') => 5,
            Key::Letter('z') => 6,
            Key::Letter('x') => 7,
            Key::Letter('c') => 8,
            Key::Letter('v') => 9,
            Key::Letter('b') => 11,
            Key::Letter('q') => 12,
            Key::Letter('w') => 13,
            Key::Letter('e') => 14,
            Key::Letter('r') => 15,
            Key::Letter('y') => 16,
            Key::Letter('t') => 17,
            Key::Num(1) => 18,
            Key::Num(2) => 19,
            Key::Num(3) => 20,
            Key::Num(4) => 21,
            Key::Num(6) => 22,
            Key::Num(5) => 23,
            Key::Num(9) => 25,
            Key::Num(7) => 26,
            Key::Num(8) => 28,
            Key::Num(0) => 29,
            Key::Letter('o') => 31,
            Key::Letter('u') => 32,
            Key::Letter('i') => 34,
            Key::Letter('p') => 35,
            Key::Letter('l') => 37,
            Key::Letter('j') => 38,
            Key::Letter('k') => 40,
            Key::Letter('n') => 45,
            Key::Letter('m') => 46,
            Key::Escape => 53,
            Key::Other(scancode) => scancode,

            Key::Num(num) => panic!("Invalid key number {num}"),
            Key::Letter(key) => panic!("Invalid key {key}"),
        }
    }

    #[cfg(target_os = "windows")]
    fn from(key: Key) -> Self {
        match key {
            Key::Escape => 1,
            Key::Num(0) => 11,
            Key::Num(num) => {
                if num < 10 {
                    num as u32 + 1
                } else {
                    panic!("Invalid key number {num}")
                }
            }
            Key::Letter('q') => 16,
            Key::Letter('w') => 17,
            Key::Letter('e') => 18,
            Key::Letter('r') => 19,
            Key::Letter('t') => 20,
            Key::Letter('y') => 21,
            Key::Letter('u') => 22,
            Key::Letter('i') => 23,
            Key::Letter('o') => 24,
            Key::Letter('p') => 25,
            Key::Letter('a') => 30,
            Key::Letter('s') => 31,
            Key::Letter('d') => 32,
            Key::Letter('f') => 33,
            Key::Letter('g') => 34,
            Key::Letter('h') => 35,
            Key::Letter('j') => 36,
            Key::Letter('k') => 37,
            Key::Letter('l') => 38,
            Key::Letter('z') => 44,
            Key::Letter('x') => 45,
            Key::Letter('c') => 46,
            Key::Letter('v') => 47,
            Key::Letter('b') => 48,
            Key::Letter('n') => 49,
            Key::Letter('m') => 50,
            Key::Letter(key) => panic!("Invalid key {key}"),
            Key::Other(scancode) => scancode,
        }
    }
}
