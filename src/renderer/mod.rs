use anyhow::Error;

use crate::remote::{App, Key, Remote};

pub mod raylib;
pub mod term;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Asset(String);

impl Asset {
    pub fn new(name: String) -> Self {
        Asset(name)
    }
    pub fn name(&self) -> &str {
        return self.0.as_str();
    }
}

pub struct Button {
    pub pos: Position,
    pub cmd: Key,
    pub shortcut: KeyCode,
    pub asset: Asset,
}

pub trait Renderer {
    fn new(buttons: [Button;17], apps: Vec<App>) -> Self;
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
