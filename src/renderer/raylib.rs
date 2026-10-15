use sola_raylib::prelude::*;

use crate::{
    remote::Remote,
    renderer::{Button, Renderer},
};

pub struct RaylibRenderer {
    handle: RaylibHandle,
    thread: RaylibThread,
    buttons: [Button; 17],
    textures: Vec<Texture2D>,
    clicked_frames: u32,
    clicked_index: usize,
}

const GRID_ROWS: i32 = 9;
const GRID_COLS: i32 = 7;
const W_WIDTH: i32 = 230;
const W_HEIGHT: i32 = 350;

include!(concat!(env!("OUT_DIR"), "/asset_map.rs"));
fn get_asset(name: &str) -> Option<&'static [u8]> {
    ASSET_MAP.iter().find(|&&(k, _)| k == name).map(|&(_, v)| v)
}

impl RaylibRenderer {}

impl Button {
    fn clicked(&self, rl: &RaylibHandle) -> bool {
        let rec = self.find_grid_position();
        if !rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && !(rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT))
        {
            return false;
        }
        let pos = rl.get_mouse_position();
        return pos.x > rec.x
            && pos.x < (rec.x + rec.width)
            && pos.y > rec.y
            && pos.y < (rec.y + rec.height);
    }

    fn find_grid_position(&self) -> Rectangle {
        let cell_width = (W_WIDTH / GRID_COLS) as f32;
        let cell_height = (W_HEIGHT / GRID_ROWS) as f32;

        return Rectangle::new(
            self.x as f32 * cell_width,
            self.y as f32 * cell_height,
            cell_width,
            cell_height,
        );
    }
}

impl Renderer for RaylibRenderer {
    fn new(buttons: [Button; 17]) -> RaylibRenderer {
        let (mut h, t) = sola_raylib::init()
            .size(W_WIDTH, W_HEIGHT)
            .title("B Controller")
            .vsync()
            .build();
        let mut assets = Vec::new();
        for b in &buttons {
            let image = Image::load_image_from_mem(".png", get_asset(&b.asset_name).unwrap()).unwrap();
            let texture = h.load_texture_from_image(&t, &image).unwrap(); //TODO: handle error
            assets.push(texture);
        }

        h.set_exit_key(None);
        return RaylibRenderer {
            handle: h,
            thread: t,
            buttons,
            textures: assets,
            clicked_frames: 0,
            clicked_index: 0,
        };
    }

    fn is_running(&self) -> bool {
        return !self.handle.window_should_close();
    }

    fn update(&mut self, remote: &mut Remote) {
        if self.clicked_frames > 0 {
            self.clicked_frames -= 1;
        }
        let key = self
            .handle
            .get_key_pressed()
            .map(|k| k as u32)
            .unwrap_or_default();
        for (i, b) in self.buttons.iter().enumerate() {
            if (b.clicked(&self.handle) && self.clicked_frames == 0) || (key == b.shortcut) {
                remote.send_key(&b.cmd).unwrap(); //TODO: Handle error
                self.clicked_frames = 15;
                self.clicked_index = i;
            }
        }
        remote.ping().unwrap();
    }

    fn draw(&mut self) {
        let mut d = self.handle.begin_drawing(&self.thread);
        let bg = Color::new(24, 24, 24, 255);
        let selected = Color::new(62, 62, 130, 255);
        d.clear_background(&bg);
        for i in 0..self.buttons.len() {
            let mut rec = self.buttons[i].find_grid_position();
            rec.width = self.textures[i].width as f32;
            rec.height = self.textures[i].height as f32;
            if self.clicked_frames > 0 && self.clicked_index == i {
                d.draw_rectangle_rounded(rec, 0.8, 16, &selected);
            } else {
                d.draw_rectangle_rounded(rec, 0.8, 16, &bg);
            }
            d.draw_texture(&self.textures[i], rec.x as i32, rec.y as i32, Color::WHITE);
        }
    }
}
