use anyhow::{Context, Error};
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
    error: Option<Error>,
    close: bool,
}

const GRID_ROWS: i32 = 9;
const GRID_COLS: i32 = 7;
const W_WIDTH: i32 = 230;
const W_HEIGHT: i32 = 350;
const W_WIDTH_ERR: i32 = 800;
const W_HEIGHT_ERR: i32 = 600;

include!(concat!(env!("OUT_DIR"), "/asset_map.rs"));
fn get_asset(name: &str) -> Option<&'static [u8]> {
    ASSET_MAP.iter().find(|&&(k, _)| k == name).map(|&(_, v)| v)
}

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
    fn display_error(&mut self, error: Error) {
        self.error = Some(error);
    }

    fn new(buttons: [Button; 17]) -> RaylibRenderer {
        let (mut h, t) = sola_raylib::init()
            .size(W_WIDTH, W_HEIGHT)
            .title("B Controller")
            .vsync()
            .build();
        let mut assets = Vec::new();
        let mut error = None;
        for b in &buttons {
            let asset = get_asset(&b.asset_name);
            match asset {
                Some(a) => {
                    let image = Image::load_image_from_mem(".png", a).with_context(|| {
                        format!("Error loading image from asset {}:", b.asset_name)
                    });
                    match image {
                        Ok(i) => {
                            let texture = h.load_texture_from_image(&t, &i).with_context(|| {
                                format!(
                                    "Error loading texture from image for asset {}:",
                                    b.asset_name
                                )
                            });
                            match texture {
                                Ok(t) => assets.push(t),
                                Err(e) => {
                                    Some(e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error = Some(e);
                            break;
                        }
                    }
                }
                None => {
                    error = Some(anyhow::anyhow!("Could not load asset {}", b.asset_name));
                    break;
                }
            }
        }

        h.set_exit_key(None);
        return RaylibRenderer {
            handle: h,
            thread: t,
            buttons,
            textures: assets,
            clicked_frames: 0,
            clicked_index: 0,
            error,
            close: false,
        };
    }

    fn is_running(&self) -> bool {
        return !self.handle.window_should_close() && !self.close;
    }

    fn update(&mut self, remote: &mut Option<Remote>) {
        match &self.error {
            Some(_) => {
                self.handle.set_window_size(W_WIDTH_ERR, W_HEIGHT_ERR);
                let btn_text_size = self.handle.measure_text("Close", 24);
                let x = (W_WIDTH_ERR / 2 - (btn_text_size / 2 + 10)) as f32;
                let y = (W_HEIGHT_ERR - 80) as f32;
                let width = (btn_text_size + 4) as f32;
                let height = 28.0;

                if self
                    .handle
                    .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                {
                    let mouse = self.handle.get_mouse_position();
                    if mouse.x > x && mouse.x < (x + width) && mouse.y > y && mouse.y < (y + height)
                    {
                        self.close = true;
                    }
                }
            }
            None => match remote {
                Some(r) => {
                    if self.clicked_frames > 0 {
                        self.clicked_frames -= 1;
                    }
                    let key = self
                        .handle
                        .get_key_pressed()
                        .map(|k| k as u32)
                        .unwrap_or_default();
                    for (i, b) in self.buttons.iter().enumerate() {
                        if (b.clicked(&self.handle) && self.clicked_frames == 0)
                            || (key == b.shortcut)
                        {
                            match r
                                .send_key(&b.cmd)
                                .with_context(|| "Failed to send key command to TV:")
                            {
                                Err(e) => {
                                    self.error = Some(e);
                                    break;
                                }
                                _ => {
                                    self.clicked_frames = 15;
                                    self.clicked_index = i;
                                }
                            }
                        }
                    }
                    if self.error.is_none() {
                        match r.ping() {
                            Err(e) => self.error = Some(e),
                            _ => {}
                        }
                    }
                }
                None => {}
            },
        }
    }

    fn draw(&mut self) {
        let bg = Color::new(24, 24, 24, 255);
        let btn_color = Color::new(8, 8, 8, 255);
        let selected = Color::new(62, 62, 130, 255);
        match &self.error {
            None => {
                let mut d = self.handle.begin_drawing(&self.thread);
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
            Some(e) => {
                let start_y = 80;
                let end_y = W_HEIGHT_ERR - 120;
                let start_x = 20;
                let end_x = W_WIDTH_ERR - 40;
                let line_width = end_x - start_x + 1;
                let line_height = 28;

                let mut lines = Vec::new();
                let error_msg = e.chain().map(|c| c.to_string()).collect::<Vec<_>>().join("\n");
                let content = error_msg.split_whitespace();
                let space_size = self.handle.measure_text(" ", 24);
                let title_size = self.handle.measure_text("Error", 32);
                let btn_text_size = self.handle.measure_text("Close", 24);

                let mut line = String::new();
                let mut remaining = line_width;
                for word in content {
                    let size = self.handle.measure_text(word, 24);
                    if size + space_size < remaining {
                        line.push_str(word);
                        line.push(' ');
                        remaining -= size;
                        remaining -= space_size;
                    } else {
                        remaining = line_width;
                        lines.push(line);
                        line = String::new();
                    }
                }
                lines.push(line);

                let mut d = self.handle.begin_drawing(&self.thread);
                d.clear_background(&bg);
                d.draw_text(
                    "Error",
                    W_WIDTH_ERR / 2 - title_size / 2,
                    20,
                    32,
                    Color::WHITE,
                );
                for (i, line) in lines.iter().enumerate() {
                    let y = start_y + (i as i32 * line_height);
                    if y > end_y {
                        break;
                    }
                    d.draw_text(
                        &line,
                        start_x,
                        y,
                        24,
                        Color::WHITE,
                    );
                }
                d.draw_rectangle(
                    W_WIDTH_ERR / 2 - (btn_text_size / 2 + 10),
                    W_HEIGHT_ERR - 80,
                    btn_text_size + 20,
                    34,
                    btn_color,
                );
                d.draw_text(
                    "Close",
                    W_WIDTH_ERR / 2 - (btn_text_size / 2),
                    W_HEIGHT_ERR - 75,
                    24,
                    Color::WHITE,
                );
            }
        }
    }
}
