use anyhow::Error;

use crate::remote::{Key, Remote};

pub mod raylib;
pub mod term;

pub struct Button {
    pub x: i32,
    pub y: i32,
    pub cmd: Key,
    pub shortcut: KeyCode,
    pub asset_name: String
}

pub trait Renderer {
    fn new(buttons: [Button;17]) -> Self;
    fn is_running(&self) -> bool;
    fn update(&mut self, remote: &mut Option<Remote>);
    fn draw(&mut self);
    fn display_error(&mut self, error: Error);
}

pub enum KeyCode {
    Equal,
    Minus,
    Up,
    Down,
    Left,
    Right,
    Space,
    Escape,
    Enter,
    M,
    P,
    H,
    J,
    K,
    Z,
    S,
    L,
}
