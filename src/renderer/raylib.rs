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
    hover_index: Option<usize>,
    error: Option<Error>,
    close: bool,
}

const GRID_ROWS: i32 = 9;
const GRID_COLS: i32 = 7;
const W_WIDTH: i32 = 280;
const W_HEIGHT: i32 = 420;
const W_WIDTH_ERR: i32 = 800;
const W_HEIGHT_ERR: i32 = 600;

const CELL_PADDING: f32 = 4.0;
const CORNER_RADIUS: f32 = 0.25;

include!(concat!(env!("OUT_DIR"), "/asset_map.rs"));
fn get_asset(name: &str) -> Option<&'static [u8]> {
    ASSET_MAP.iter().find(|&&(k, _)| k == name).map(|&(_, v)| v)
}

fn button_group(index: usize) -> u8 {
    match index {
        0 => 0, // Power
        1..=3 => 1, // Volume group
        4..=6 => 2, // Channel group
        7..=11 => 3, // Navigation group
        12..=14 => 4, // Back/Enter/Play
        15..=16 => 6, // Home/Menu
        _ => 7,
    }
}

fn group_color(group: u8) -> Color {
    match group {
        0 => Color::new(180, 40, 40, 255),    // Power - red accent
        1 => Color::new(40, 120, 180, 255),   // Volume - blue
        2 => Color::new(180, 120, 40, 255),   // Channel - amber
        3 => Color::new(60, 140, 80, 255),    // Navigation - green
        4 => Color::new(140, 60, 140, 255),   // Back/Enter/Play - purple
        5 => Color::new(40, 140, 140, 255),   // Play/Pause - cyan
        6 => Color::new(120, 120, 120, 255),  // Home/Menu - neutral
        _ => Color::new(80, 80, 80, 255),
    }
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

    fn is_hovered(&self, rl: &RaylibHandle) -> bool {
        let rec = self.find_grid_position();
        let pos = rl.get_mouse_position();
        pos.x > rec.x && pos.x < (rec.x + rec.width) && pos.y > rec.y && pos.y < (rec.y + rec.height)
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

    fn icon_rect(&self) -> Rectangle {
        let cell = self.find_grid_position();
        // Source icons are 24x24; don't upscale beyond that to avoid pixelation
        let max_icon_size = 24.0;
        let icon_size = (cell.width.min(cell.height) - CELL_PADDING * 2.0).min(max_icon_size).max(16.0);
        Rectangle::new(
            cell.x + (cell.width - icon_size) / 2.0,
            cell.y + (cell.height - icon_size) / 2.0,
            icon_size,
            icon_size,
        )
    }
}

impl Renderer for RaylibRenderer {
    fn display_error(&mut self, error: Error) {
        self.error = Some(error);
    }

    fn new(buttons: [Button; 17]) -> RaylibRenderer {
        let (mut h, t) = sola_raylib::init()
            .size(W_WIDTH, W_HEIGHT)
            .title("bController")
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
                                    error = Some(e);
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
            hover_index: None,
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

                    // Update hover state
                    self.hover_index = None;
                    for (i, b) in self.buttons.iter().enumerate() {
                        if b.is_hovered(&self.handle) {
                            self.hover_index = Some(i);
                            break;
                        }
                    }

                    // Handle shortcut keys and mouse clicks
                    let key = self
                        .handle
                        .get_key_pressed()
                        .map(|k| k as u32)
                        .unwrap_or_default();

                    for (i, b) in self.buttons.iter().enumerate() {
                        let mouse_clicked = b.clicked(&self.handle) && self.clicked_frames == 0;
                        let shortcut_pressed = key == b.shortcut;

                        if mouse_clicked || shortcut_pressed {
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
        let bg = Color::new(18, 18, 20, 255);
        let btn_base = Color::new(28, 28, 32, 255);
        let btn_hover = Color::new(40, 40, 48, 255);

        match &self.error {
            None => {
                let mut d = self.handle.begin_drawing(&self.thread);
                d.clear_background(&bg);

                // Draw subtle grid group backgrounds
                for i in 0..self.buttons.len() {
                    let cell = self.buttons[i].find_grid_position();
                    let group = button_group(i);
                    let group_col = group_color(group);
                    let group_bg = Color::new(
                        (group_col.r as f32 * 0.08 + btn_base.r as f32 * 0.92) as u8,
                        (group_col.g as f32 * 0.08 + btn_base.g as f32 * 0.92) as u8,
                        (group_col.b as f32 * 0.08 + btn_base.b as f32 * 0.92) as u8,
                        255,
                    );
                    d.draw_rectangle_rounded(
                        Rectangle::new(cell.x + 1.0, cell.y + 1.0, cell.width - 2.0, cell.height - 2.0),
                        CORNER_RADIUS,
                        8,
                        &group_bg,
                    );
                }

                // Draw buttons
                for i in 0..self.buttons.len() {
                    let cell = self.buttons[i].find_grid_position();
                    let icon_rect = self.buttons[i].icon_rect();
                    let group = button_group(i);
                    let group_col = group_color(group);

                    let is_clicked = self.clicked_frames > 0 && self.clicked_index == i;
                    let is_hovered = self.hover_index == Some(i);

                    // Button background
                    let bg_color = if is_clicked {
                        Color::new(
                            (group_col.r as f32 * 0.4 + btn_base.r as f32 * 0.6) as u8,
                            (group_col.g as f32 * 0.4 + btn_base.g as f32 * 0.6) as u8,
                            (group_col.b as f32 * 0.4 + btn_base.b as f32 * 0.6) as u8,
                            255,
                        )
                    } else if is_hovered {
                        btn_hover
                    } else {
                        btn_base
                    };

                    d.draw_rectangle_rounded(
                        Rectangle::new(cell.x + 1.0, cell.y + 1.0, cell.width - 2.0, cell.height - 2.0),
                        CORNER_RADIUS,
                        8,
                        &bg_color,
                    );

                    // Subtle group accent line at top of button
                    if !is_clicked {
                        let accent_alpha = if is_hovered { 255 } else { 100 };
                        d.draw_rectangle_rounded(
                            Rectangle::new(cell.x + 1.0, cell.y + 1.0, cell.width - 2.0, 2.0),
                            0.5,
                            4,
                            &Color::new(group_col.r, group_col.g, group_col.b, accent_alpha),
                        );
                    }

                    // Draw icon centered in cell
                    if i < self.textures.len() {
                        let tex = &self.textures[i];
                        let tint = if is_clicked {
                            Color::new(220, 220, 230, 255)
                        } else {
                            Color::WHITE
                        };
                        d.draw_texture_pro(
                            tex,
                            Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32),
                            icon_rect,
                            Vector2::new(0.0, 0.0),
                            0.0,
                            tint,
                        );
                    }
                }

            }
            Some(e) => {
                let start_y = 80;
                let end_y = W_HEIGHT_ERR - 120;
                let start_x = 20;
                let end_x = W_WIDTH_ERR - 40;
                let line_height = 28;

                let error_msg = e.chain().map(|c| c.to_string()).collect::<Vec<_>>().join("\n");
                let lines_raw: Vec<&str> = error_msg.lines().collect();
                let mut lines = Vec::new();
                let space_size = self.handle.measure_text(" ", 24);
                let title_size = self.handle.measure_text("Error", 32);
                let btn_text_size = self.handle.measure_text("Close", 24);
                let line_width = end_x - start_x + 1;

                for raw_line in lines_raw {
                    let content = raw_line.split_whitespace();
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
                            line.push_str(word);
                            line.push(' ');
                            remaining -= size;
                            remaining -= space_size;
                        }
                    }
                    if !line.is_empty() {
                        lines.push(line);
                    } else {
                        lines.push(String::new()); // preserve empty lines
                    }
                }

                let mut d = self.handle.begin_drawing(&self.thread);
                d.clear_background(&bg);
                d.draw_text(
                    "Error",
                    W_WIDTH_ERR / 2 - title_size / 2,
                    20,
                    32,
                    Color::new(220, 80, 80, 255),
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
                    btn_base,
                );
                d.draw_rectangle_rounded_lines_ex(
                    Rectangle::new(
                        (W_WIDTH_ERR / 2 - (btn_text_size / 2 + 10)) as f32,
                        (W_HEIGHT_ERR - 80) as f32,
                        (btn_text_size + 20) as f32,
                        34.0,
                    ),
                    0.3,
                    8,
                    2.0,
                    &Color::new(100, 100, 120, 255),
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