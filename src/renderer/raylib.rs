use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use anyhow::{Context, Error, Result, bail};
use sola_raylib::{prelude::*};

use crate::{
    remote::{App, Key, Remote},
    renderer::{Asset, Button, KeyCode, Position, Renderer},
};

pub struct RaylibRenderer {
    handle: RaylibHandle,
    thread: RaylibThread,
    buttons: Vec<DrawableButton>,
    font: HashMap<i32, Font>,
    clicked_frames: u32,
    clicked_index: usize,
    hover_index: Option<usize>,
    error: Option<Error>,
    close: bool,
}

const GRID_ROWS: i32 = 9;
const GRID_COLS: i32 = 7;
const W_WIDTH_CONTROLS: i32 = 280;
const W_WIDTH: i32 = W_WIDTH_CONTROLS * 2;
const W_HEIGHT: i32 = 420;
const W_WIDTH_ERR: i32 = 800;
const W_HEIGHT_ERR: i32 = 600;

const CELL_PADDING: f32 = 4.0;
const CORNER_RADIUS: f32 = 0.25;

include!(concat!(env!("OUT_DIR"), "/asset_map.rs"));
impl Asset {
    fn get_bytes(&self) -> Option<&'static [u8]> {
        let name = &self.0;
        ASSET_MAP.iter().find(|&&(k, _)| k == name).map(|&(_, v)| v)
    }
    fn get_texture(&self, t: &RaylibThread, h: &mut RaylibHandle) -> Result<Texture2D> {
        let asset = &self.get_bytes();
        match asset {
            None => bail!("Could not load asset {}", self.name()),
            Some(a) => {
                let image = Image::load_image_from_mem(".png", a)
                    .with_context(|| format!("Error loading image from asset {}:", self.name()))?;
                let texture = h.load_texture_from_image(&t, &image).with_context(|| {
                    format!(
                        "Error loading texture from image for asset {}:",
                        self.name()
                    )
                })?;
                texture.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_ANISOTROPIC_16X);
                return Ok(texture);
            }
        }
    }
    fn default_fonts(h: &mut RaylibHandle) -> HashMap<i32, Font>{
        let mut fonts = HashMap::new();
        let def24 = unsafe { Font::from_raw(h.get_font_default().to_owned()) };
        let def32 = unsafe { Font::from_raw(h.get_font_default().to_owned()) };
        fonts.insert(24, def24);
        fonts.insert(32, def32);
        return fonts;
    }

    fn get_font(&self, t: &RaylibThread, h: &mut RaylibHandle) -> Result<HashMap<i32, Font>> {
        let asset = &self.get_bytes();
        let mut fonts = HashMap::with_capacity(2);
        match asset {
            None => bail!("Could not load asset {}", self.name()),
            Some(a) => {
                let chars: String = h
                    .get_font_default()
                    .chars()
                    .iter()
                    .map(|glyph| glyph.value as u8 as char)
                    .collect();
                let font24 = h.load_font_from_memory(t, ".ttf", *a, 24, Some(chars.as_ref()))?;
                let font32 = h.load_font_from_memory(t, ".ttf", *a, 48, Some(chars.as_ref()))?;
                fonts.insert(24, font24);
                fonts.insert(32, font32);
                return Ok(fonts);
            }
        };
    }
}

struct ButtonGroup(u8);

impl ButtonGroup {
    pub fn new(index: usize) -> Self {
        let val = match index {
            0 => 0,       // Power
            1..=3 => 1,   // Volume group
            4..=6 => 2,   // Channel group
            7..=11 => 3,  // Navigation group
            12..=14 => 4, // Back/Enter/Play
            15..=16 => 6, // Home/Menu
            _ => 7,
        };
        return ButtonGroup(val);
    }

    pub fn color(&self) -> Color {
        match self.0 {
            0 => Color::new(180, 40, 40, 255),   // Power - red accent
            1 => Color::new(40, 120, 180, 255),  // Volume - blue
            2 => Color::new(180, 120, 40, 255),  // Channel - amber
            3 => Color::new(60, 140, 80, 255),   // Navigation - green
            4 => Color::new(140, 60, 140, 255),  // Back/Enter/Play - purple
            5 => Color::new(40, 140, 140, 255),  // Play/Pause - cyan
            6 => Color::new(120, 120, 120, 255), // Home/Menu - neutral
            _ => Color::new(80, 80, 80, 255),
        }
    }
}

impl KeyCode {
    pub fn to_raylib(&self) -> KeyboardKey {
        match self {
            KeyCode::Equal => KeyboardKey::KEY_EQUAL,
            KeyCode::Minus => KeyboardKey::KEY_MINUS,
            KeyCode::Up => KeyboardKey::KEY_UP,
            KeyCode::Down => KeyboardKey::KEY_DOWN,
            KeyCode::Left => KeyboardKey::KEY_LEFT,
            KeyCode::Right => KeyboardKey::KEY_RIGHT,
            KeyCode::Space => KeyboardKey::KEY_SPACE,
            KeyCode::Escape => KeyboardKey::KEY_ESCAPE,
            KeyCode::Enter => KeyboardKey::KEY_ENTER,
            KeyCode::M => KeyboardKey::KEY_M,
            KeyCode::P => KeyboardKey::KEY_P,
            KeyCode::H => KeyboardKey::KEY_H,
            KeyCode::J => KeyboardKey::KEY_J,
            KeyCode::K => KeyboardKey::KEY_K,
            KeyCode::Z => KeyboardKey::KEY_Z,
            KeyCode::S => KeyboardKey::KEY_S,
            KeyCode::L => KeyboardKey::KEY_L,
        }
    }
}

impl Position {
    fn find_grid_position(&self) -> Rectangle {
        let cell_width = (W_WIDTH_CONTROLS / GRID_COLS) as f32;
        let cell_height = (W_HEIGHT / GRID_ROWS) as f32;

        return Rectangle::new(
            self.x as f32 * cell_width,
            self.y as f32 * cell_height,
            cell_width,
            cell_height,
        );
    }
}

struct DrawableButton {
    cell: Rectangle,
    group: ButtonGroup,
    texture: Texture2D,
    max_icon_size: f32,
    key: Option<Key>,
    app: Option<App>,
    shortcut: Option<KeyboardKey>,
    action: fn(remote: &mut Remote, btn: &DrawableButton) -> Result<()>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);
impl DrawableButton {
    fn from_button(button: &Button, group: ButtonGroup, texture: Texture2D) -> Self {
        let cell = button.pos.find_grid_position();
        let action = |remote: &mut Remote, btn: &DrawableButton| {
            remote
                .send_key(&btn.key.unwrap())
                .with_context(|| "Failed to send key command to TV:")?;
            Ok(())
        };
        let shortcut = button.shortcut.to_raylib();
        DrawableButton {
            cell,
            group,
            texture,
            shortcut: Some(shortcut),
            key: Some(button.cmd),
            app: None,
            max_icon_size: 24.0,
            action: action,
        }
    }

    fn from_app(app: &App, group: ButtonGroup, texture: Texture2D) -> Self {
        let cell = App::find_grid_position(COUNTER.fetch_add(1, SeqCst));
        let action = |remote: &mut Remote, btn: &DrawableButton| {
            let _ = remote.open_app(btn.app.unwrap())?;
            Ok(())
        };
        DrawableButton {
            cell,
            group,
            texture,
            shortcut: None,
            key: None,
            app: Some(*app),
            max_icon_size: 50.0,
            action: action,
        }
    }

    fn click(&self, remote: &mut Remote) -> Result<()> {
        return (self.action)(remote, self);
    }

    fn clicked(&self, rl: &RaylibHandle) -> bool {
        if !rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && !(rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT))
        {
            return false;
        }
        return self.is_hovered(rl);
    }

    fn is_hovered(&self, rl: &RaylibHandle) -> bool {
        let rec = self.cell;
        let pos = rl.get_mouse_position();
        pos.x > rec.x
            && pos.x < (rec.x + rec.width)
            && pos.y > rec.y
            && pos.y < (rec.y + rec.height)
    }

    fn icon_rect(&self) -> Rectangle {
        let icon_size = (self.cell.width.min(self.cell.height) - CELL_PADDING * 2.0)
            .min(self.max_icon_size)
            .max(16.0);
        Rectangle::new(
            self.cell.x + (self.cell.width - icon_size) / 2.0,
            self.cell.y + (self.cell.height - icon_size) / 2.0,
            icon_size,
            icon_size,
        )
    }
    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        btn_base: Color,
        btn_hover: Color,
        is_clicked: bool,
        is_hovered: bool,
    ) {
        //Draw bg
        let group_col = self.group.color();
        let group_bg = Color::new(
            (group_col.r as f32 * 0.08 + btn_base.r as f32 * 0.92) as u8,
            (group_col.g as f32 * 0.08 + btn_base.g as f32 * 0.92) as u8,
            (group_col.b as f32 * 0.08 + btn_base.b as f32 * 0.92) as u8,
            255,
        );
        d.draw_rectangle_rounded(
            Rectangle::new(
                self.cell.x + 1.0,
                self.cell.y + 1.0,
                self.cell.width - 2.0,
                self.cell.height - 2.0,
            ),
            CORNER_RADIUS,
            8,
            &group_bg,
        );

        let group_col = self.group.color();
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
            Rectangle::new(
                self.cell.x + 1.0,
                self.cell.y + 1.0,
                self.cell.width - 2.0,
                self.cell.height - 2.0,
            ),
            CORNER_RADIUS,
            8,
            &bg_color,
        );

        // Subtle group accent line at top of button
        if !is_clicked {
            let accent_alpha = if is_hovered { 255 } else { 100 };
            d.draw_rectangle_rounded(
                Rectangle::new(
                    self.cell.x + 1.0,
                    self.cell.y + 1.0,
                    self.cell.width - 2.0,
                    2.0,
                ),
                0.5,
                4,
                &Color::new(group_col.r, group_col.g, group_col.b, accent_alpha),
            );
        }

        // Draw icon centered in cell
        let tint = if is_clicked {
            Color::new(220, 220, 230, 255)
        } else {
            Color::WHITE
        };
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.texture.width as f32,
                self.texture.height as f32,
            ),
            self.icon_rect(),
            Vector2::new(0.0, 0.0),
            0.0,
            tint,
        );
    }
}

impl App {
    fn find_grid_position(index: usize) -> Rectangle {
        let cell_width = 68.0;
        let cell_height = 68.0;

        let row_count = ((W_WIDTH_CONTROLS - 20) as f32 / (cell_width + 10.0)) as usize;
        let row_size = row_count as f32 * (cell_width + 5.0);
        let start_x = W_WIDTH_CONTROLS as f32 / 2.0 - row_size / 2.0;
        let y = 40.0 + ((index / row_count) as f32 * (cell_height + 5.0));
        let x = W_WIDTH_CONTROLS as f32
            + start_x
            + ((index % row_count as usize) as f32 * (cell_width + 10.0));
        return Rectangle::new(x, y, cell_width, cell_height);
    }
}

impl Renderer for RaylibRenderer {
    fn display_error(&mut self, error: Error) {
        self.error = Some(error);
    }

    fn new(buttons: [Button; 17], apps: Vec<App>) -> RaylibRenderer {
        let (mut h, t) = sola_raylib::init()
            .size(W_WIDTH, W_HEIGHT)
            .title("b-remote")
            .vsync()
            .build();
        let mut error = None;
        let mut btns = Vec::new();
        for (i, b) in buttons.iter().enumerate() {
            let texture = b.asset.get_texture(&t, &mut h);
            match texture {
                Ok(tex) => {
                    let group = ButtonGroup::new(i);
                    btns.push(DrawableButton::from_button(b, group, tex));
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        for app in apps {
            let asset = Asset::new(app.name().into());
            let texture = asset.get_texture(&t, &mut h);
            match texture {
                Ok(tex) => {
                    let group = ButtonGroup::new(8);
                    btns.push(DrawableButton::from_app(&app, group, tex));
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }

        let font_asset = Asset::new("font".into());
        let font = match font_asset.get_font(&t, &mut h) {
            Err(e) => {
                error = Some(e);
                Asset::default_fonts(&mut h)
            }
            Ok(font) => font,
        };
        h.set_exit_key(None);
        return RaylibRenderer {
            handle: h,
            thread: t,
            buttons: btns,
            font,
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
                let font = self.font.get(&24).unwrap();
                self.handle.set_window_size(W_WIDTH_ERR, W_HEIGHT_ERR);
                let btn_text_size = font.measure_text("Close", 24.0, 0.5);
                let x = W_WIDTH_ERR as f32 / 2.0 - (btn_text_size.x / 2.0 + 10.0);
                let y = (W_HEIGHT_ERR - 80) as f32;
                let width = btn_text_size.x + 4.0;
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
                        .unwrap_or(KeyboardKey::KEY_NULL);

                    for (i, b) in self.buttons.iter().enumerate() {
                        let mouse_clicked = b.clicked(&self.handle) && self.clicked_frames == 0;
                        let shortcut_pressed = b.shortcut.is_some() && key == b.shortcut.unwrap();

                        if mouse_clicked || shortcut_pressed {
                            match b.click(r) {
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

        let font32 = self.font.get(&32).unwrap();
        let font24 = self.font.get(&24).unwrap();
        match &self.error {
            None => {
                let title_size = font24.measure_text("Applications", 24.0, 0.5);
                let mut d = self.handle.begin_drawing(&self.thread);
                d.clear_background(&bg);

                for i in 0..self.buttons.len() {
                    let is_clicked = self.clicked_frames > 0 && self.clicked_index == i;
                    let is_hovered = self.hover_index == Some(i);
                    self.buttons[i].draw(&mut d, btn_base, btn_hover, is_clicked, is_hovered);
                }

                let title_x = W_WIDTH_CONTROLS / 2 - title_size.x as i32 / 2;
                d.draw_text_ex(
                    &font24,
                    "Applications",
                    Vector2::new((W_WIDTH_CONTROLS + title_x) as f32, 10.0),
                    24.0,
                    0.5,
                    Color::RAYWHITE,
                );
            }
            Some(e) => {
                let start_y = 80;
                let end_y = W_HEIGHT_ERR - 120;
                let start_x = 20;
                let end_x = W_WIDTH_ERR - 40;

                let error_msg = e
                    .chain()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                let lines_raw: Vec<&str> = error_msg.lines().collect();
                let mut lines = Vec::new();
                let space_size = font24.measure_text(" ", 24.0, 0.5);
                let title_size = font32.measure_text("Error", 32.0, 0.5);
                let btn_text_size = font24.measure_text("Close", 24.0, 0.5);
                let line_width = end_x - start_x + 1;

                for raw_line in lines_raw {
                    let content = raw_line.split_whitespace();
                    let mut line = String::new();
                    let mut remaining = line_width as f32;
                    for word in content {
                        let size = font24.measure_text(word, 24.0, 0.5);
                        if size.x + space_size.x < remaining {
                            line.push_str(word);
                            line.push(' ');
                            remaining -= size.x;
                            remaining -= space_size.x;
                        } else {
                            remaining = line_width as f32;
                            lines.push(line);
                            line = String::new();
                            line.push_str(word);
                            line.push(' ');
                            remaining -= size.x;
                            remaining -= space_size.x;
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
                d.draw_text_ex(
                    &font32,
                    "Error",
                    Vector2::new(W_WIDTH_ERR as f32 / 2.0 - title_size.x / 2.0, 20.0),
                    32.0,
                    0.5,
                    Color::new(220, 80, 80, 255),
                );
                for (i, line) in lines.iter().enumerate() {
                    let y = start_y + (i as i32 * space_size.y as i32);
                    if y > end_y {
                        break;
                    }
                    d.draw_text_ex(
                        &font24,
                        &line,
                        Vector2::new(start_x as f32, y as f32),
                        24.0,
                        0.5,
                        Color::WHITE,
                    );
                }
                d.draw_rectangle(
                    W_WIDTH_ERR / 2 - (btn_text_size.x as i32 / 2 + 10),
                    W_HEIGHT_ERR - 80,
                    btn_text_size.x as i32 + 20,
                    34,
                    btn_base,
                );
                d.draw_rectangle_rounded_lines_ex(
                    Rectangle::new(
                        W_WIDTH_ERR as f32 / 2.0 - (btn_text_size.x / 2.0 + 10.0),
                        (W_HEIGHT_ERR - 80) as f32,
                        btn_text_size.x + 20.0,
                        34.0,
                    ),
                    0.3,
                    8,
                    2.0,
                    &Color::new(100, 100, 120, 255),
                );
                d.draw_text_ex(
                    &font24,
                    "Close",
                    Vector2::new(
                        W_WIDTH_ERR as f32 / 2.0 - (btn_text_size.x / 2.0),
                        (W_HEIGHT_ERR - 75) as f32,
                    ),
                    24.0,
                    0.5,
                    Color::WHITE,
                );
            }
        }
    }
}
