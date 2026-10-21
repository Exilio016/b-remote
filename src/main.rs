use anyhow::Result;
use crate::remote::{Key, Remote};
use crate::renderer::term::TermRenderer;
use crate::renderer::{Asset, Button, KeyCode, Position, Renderer};
use crate::renderer::raylib::RaylibRenderer;

mod remote;
mod renderer;

fn app(renderer:&mut impl Renderer, remote:Result<Remote>) {
    let mut r = None;
    match remote {
        Err(e) => {
            renderer.display_error(e);
        }
        Ok(rem) => r = Some(rem) 
    }
    while renderer.is_running() {
        renderer.update(&mut r);
        renderer.draw();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let remote = Remote::new();
    let buttons: [Button; 17] = [
        Button { pos: Position {x: 0, y: 0}, cmd: Key::Power,   shortcut: KeyCode::P, asset: Asset::new("circle-power".into())},
        Button { pos: Position {x: 0, y: 2}, cmd: Key::VolUp,   shortcut: KeyCode::Equal, asset: Asset::new("arrow-up".into())},
        Button { pos: Position {x: 0, y: 3}, cmd: Key::Mute,    shortcut: KeyCode::M, asset: Asset::new("volume-x".into())},
        Button { pos: Position {x: 0, y: 4}, cmd: Key::VolDown, shortcut: KeyCode::Minus, asset: Asset::new("arrow-down".into())},
        Button { pos: Position {x: 6, y: 2}, cmd: Key::ChUp,    shortcut: KeyCode::K, asset: Asset::new("arrow-up".into())},
        Button { pos: Position {x: 6, y: 3}, cmd: Key::ChList,  shortcut: KeyCode::L, asset: Asset::new("tv".into())},
        Button { pos: Position {x: 6, y: 4}, cmd: Key::ChDown,  shortcut: KeyCode::J, asset: Asset::new("arrow-down".into())},
        Button { pos: Position {x: 3, y: 2}, cmd: Key::Up,      shortcut: KeyCode::Up, asset: Asset::new("circle-arrow-up".into())},
        Button { pos: Position {x: 2, y: 3}, cmd: Key::Left,    shortcut: KeyCode::Left, asset: Asset::new("circle-arrow-left".into())},
        Button { pos: Position {x: 3, y: 3}, cmd: Key::Enter,   shortcut: KeyCode::Enter, asset: Asset::new("circle".into())},
        Button { pos: Position {x: 4, y: 3}, cmd: Key::Right,   shortcut: KeyCode::Right, asset: Asset::new("circle-arrow-right".into())},
        Button { pos: Position {x: 3, y: 4}, cmd: Key::Down,    shortcut: KeyCode::Down, asset: Asset::new("circle-arrow-down".into())},
        Button { pos: Position {x: 2, y: 6}, cmd: Key::Back,    shortcut: KeyCode::Escape, asset: Asset::new("undo-2".into())},
        Button { pos: Position {x: 3, y: 6}, cmd: Key::Play,    shortcut: KeyCode::Space, asset: Asset::new("play".into())},
        Button { pos: Position {x: 4, y: 6}, cmd: Key::Pause,   shortcut: KeyCode::Z, asset: Asset::new("pause".into())},
        Button { pos: Position {x: 2, y: 8}, cmd: Key::Home,    shortcut: KeyCode::H, asset: Asset::new("house".into())},
        Button { pos: Position {x: 4, y: 8}, cmd: Key::Menu,    shortcut: KeyCode::S, asset: Asset::new("settings".into())}
    ];
    if args.contains(&"--tui".to_owned()) {
        let mut r = TermRenderer::new(buttons);
        app(&mut r, remote);
    }
    else {
        let mut r = RaylibRenderer::new(buttons);
        app(&mut r, remote);
    }
}
