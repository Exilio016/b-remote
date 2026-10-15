use sola_raylib::ffi::KeyboardKey;

use crate::remote::{Key, Remote};
use crate::renderer::term::TermRenderer;
use crate::renderer::{Button, Renderer};
use crate::renderer::raylib::RaylibRenderer;

mod remote;
mod renderer;

fn app(renderer:&mut impl Renderer, remote: &mut Remote) {
    while renderer.is_running() {
        renderer.update(remote);
        renderer.draw();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut remote = Remote::new().unwrap();
    let buttons: [Button; 17] = [
        Button { x: 0, y: 0, cmd: Key::Power,   shortcut: KeyboardKey::KEY_P as u32, asset_name: "circle-power".into()},
        Button { x: 0, y: 2, cmd: Key::VolUp,   shortcut: KeyboardKey::KEY_EQUAL as u32, asset_name: "arrow-up".into()},
        Button { x: 0, y: 3, cmd: Key::Mute,    shortcut: KeyboardKey::KEY_M as u32, asset_name: "volume-x".into()},
        Button { x: 0, y: 4, cmd: Key::VolDown, shortcut: KeyboardKey::KEY_MINUS as u32, asset_name: "arrow-down".into()},
        Button { x: 6, y: 2, cmd: Key::ChUp,    shortcut: KeyboardKey::KEY_K as u32, asset_name: "arrow-up".into()},
        Button { x: 6, y: 3, cmd: Key::ChList,  shortcut: KeyboardKey::KEY_G as u32, asset_name: "tv".into()},
        Button { x: 6, y: 4, cmd: Key::ChDown,  shortcut: KeyboardKey::KEY_J as u32, asset_name: "arrow-down".into()},
        Button { x: 3, y: 2, cmd: Key::Up,      shortcut: KeyboardKey::KEY_UP as u32, asset_name: "circle-arrow-up".into()},
        Button { x: 2, y: 3, cmd: Key::Left,    shortcut: KeyboardKey::KEY_LEFT as u32, asset_name: "circle-arrow-left".into()},
        Button { x: 3, y: 3, cmd: Key::Enter,   shortcut: KeyboardKey::KEY_ENTER as u32, asset_name: "circle".into()},
        Button { x: 4, y: 3, cmd: Key::Right,   shortcut: KeyboardKey::KEY_RIGHT as u32, asset_name: "circle-arrow-right".into()},
        Button { x: 3, y: 4, cmd: Key::Down,    shortcut: KeyboardKey::KEY_DOWN as u32, asset_name: "circle-arrow-down".into()},
        Button { x: 2, y: 6, cmd: Key::Back,    shortcut: KeyboardKey::KEY_ESCAPE as u32, asset_name: "undo-2".into()},
        Button { x: 3, y: 6, cmd: Key::Play,    shortcut: KeyboardKey::KEY_SPACE as u32, asset_name: "play".into()},
        Button { x: 4, y: 6, cmd: Key::Pause,   shortcut: KeyboardKey::KEY_Z as u32, asset_name: "pause".into()},
        Button { x: 2, y: 8, cmd: Key::Home,    shortcut: KeyboardKey::KEY_H as u32, asset_name: "house".into()},
        Button { x: 4, y: 8, cmd: Key::Menu,    shortcut: KeyboardKey::KEY_S as u32, asset_name: "settings".into()}
    ];
    if args.contains(&"--tui".to_owned()) {
        let mut r = TermRenderer::new(buttons);
        app(&mut r, &mut remote);
    }
    else {
        let mut r = RaylibRenderer::new(buttons);
        app(&mut r, &mut remote);
    }
}
