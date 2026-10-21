use anyhow::Result;
use crate::remote::{Key, Remote};
use crate::renderer::term::TermRenderer;
use crate::renderer::{Button, KeyCode, Renderer};
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
        Button { x: 0, y: 0, cmd: Key::Power,   shortcut: KeyCode::P, asset_name: "circle-power".into()},
        Button { x: 0, y: 2, cmd: Key::VolUp,   shortcut: KeyCode::Equal, asset_name: "arrow-up".into()},
        Button { x: 0, y: 3, cmd: Key::Mute,    shortcut: KeyCode::M, asset_name: "volume-x".into()},
        Button { x: 0, y: 4, cmd: Key::VolDown, shortcut: KeyCode::Minus, asset_name: "arrow-down".into()},
        Button { x: 6, y: 2, cmd: Key::ChUp,    shortcut: KeyCode::K, asset_name: "arrow-up".into()},
        Button { x: 6, y: 3, cmd: Key::ChList,  shortcut: KeyCode::L, asset_name: "tv".into()},
        Button { x: 6, y: 4, cmd: Key::ChDown,  shortcut: KeyCode::J, asset_name: "arrow-down".into()},
        Button { x: 3, y: 2, cmd: Key::Up,      shortcut: KeyCode::Up, asset_name: "circle-arrow-up".into()},
        Button { x: 2, y: 3, cmd: Key::Left,    shortcut: KeyCode::Left, asset_name: "circle-arrow-left".into()},
        Button { x: 3, y: 3, cmd: Key::Enter,   shortcut: KeyCode::Enter, asset_name: "circle".into()},
        Button { x: 4, y: 3, cmd: Key::Right,   shortcut: KeyCode::Right, asset_name: "circle-arrow-right".into()},
        Button { x: 3, y: 4, cmd: Key::Down,    shortcut: KeyCode::Down, asset_name: "circle-arrow-down".into()},
        Button { x: 2, y: 6, cmd: Key::Back,    shortcut: KeyCode::Escape, asset_name: "undo-2".into()},
        Button { x: 3, y: 6, cmd: Key::Play,    shortcut: KeyCode::Space, asset_name: "play".into()},
        Button { x: 4, y: 6, cmd: Key::Pause,   shortcut: KeyCode::Z, asset_name: "pause".into()},
        Button { x: 2, y: 8, cmd: Key::Home,    shortcut: KeyCode::H, asset_name: "house".into()},
        Button { x: 4, y: 8, cmd: Key::Menu,    shortcut: KeyCode::S, asset_name: "settings".into()}
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
