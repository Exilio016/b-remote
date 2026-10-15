use anyhow::Error;
use crosscurses::{ALL_MOUSE_EVENTS, Input, MEVENT, REPORT_MOUSE_POSITION, Window, initscr};
use std::{collections::HashMap, ptr::null_mut};

use crate::{remote::Remote, renderer::Renderer};

use super::Button;

pub struct TermRenderer {
    buttons: [Button; 17],
    assets: HashMap<String, char>,
    running: bool,
    window: Window,
    error: Option<Error>,
}

const GRID_ROWS: f32 = 9.0;
const GRID_COLS: f32 = 7.0;
const W_WIDTH: f32 = 15.0;
const W_HEIGHT: f32 = 8.0;

impl Button {
    fn find_grid_pos(&self) -> (i32, i32) {
        let cell_width = W_WIDTH / GRID_COLS;
        let cell_height = W_HEIGHT / GRID_ROWS;
        return (
            2 + (self.x as f32 * cell_width) as i32,
            3 + (self.y as f32 * cell_height) as i32,
        );
    }
}

impl TermRenderer {
    fn get_keycode(&self) -> u32 {
        let key = self.window.getch().unwrap();
        let keycode = match key {
            Input::Character(c) => {
                if c == '\n' {
                    257
                } else if c == '\u{1b}' {
                    256
                } else if c == '+' {
                    '=' as u32
                } else if c == '_' {
                    '-' as u32
                } else {
                    c.to_ascii_uppercase() as u32
                }
            }
            Input::KeyUp => 265,
            Input::KeyLeft => 263,
            Input::KeyEnter => 257,
            Input::KeyRight => 262,
            Input::KeyDown => 264,
            Input::KeyMouse => 1,
            _ => 0,
        };
        return keycode;
    }
}

impl Renderer for TermRenderer {
    fn new(buttons: [Button; 17]) -> Self {
        let assets = HashMap::from([
            ("circle-power".to_owned(), '⏻'),
            ("arrow-up".to_owned(), ''),
            ("volume-x".to_owned(), '󰝟'),
            ("arrow-down".to_owned(), ''),
            ("tv".to_owned(), '󰑈'),
            ("circle-arrow-up".to_owned(), ''),
            ("circle-arrow-left".to_owned(), ''),
            ("circle".to_owned(), ''),
            ("circle-arrow-right".to_owned(), ''),
            ("circle-arrow-down".to_owned(), ''),
            ("undo-2".to_owned(), ''),
            ("play".to_owned(), ''),
            ("pause".to_owned(), ''),
            ("house".to_owned(), ''),
            ("settings".to_owned(), ''),
        ]);
        let window = initscr();
        crosscurses::curs_set(0);
        window.keypad(true);
        crosscurses::mousemask(ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION, null_mut());
        window.printw("Press q to quit...\n");
        let mut term = TermRenderer {
            buttons,
            assets,
            running: true,
            window,
            error: None,
        };
        term.draw();
        return term;
    }

    fn is_running(&self) -> bool {
        return self.running;
    }

    fn update(&mut self, remote: &mut Option<Remote>) {
        let keycode = self.get_keycode();
        let mut mevent: Option<MEVENT> = None;
        if keycode == 1 {
            mevent = Some(crosscurses::getmouse().unwrap());
        }
        if keycode == 'Q' as u32 {
            self.running = false;
        }
        if self.error.is_none() {
            match remote {
                Some(remote) => {
                    for b in &self.buttons {
                        let (x, y) = b.find_grid_pos();
                        if keycode == b.shortcut
                            || (mevent.is_some()
                                && x == mevent.unwrap().x
                                && y == mevent.unwrap().y)
                        {
                            remote.send_key(&b.cmd).unwrap();
                        }
                    }
                    remote.ping().unwrap();
                }
                None => {}
            }
        }
    }

    fn draw(&mut self) {
        if !self.running {
            self.window.refresh();
            crosscurses::curs_set(1);
            crosscurses::endwin();
            return;
        }
        match &self.error {
            None => {
                let border = r#"┌────────────────┐
│                │
│                │
│                │
│                │
│                │
│                │
│                │
│                │
│                │
└────────────────┘"#;
                self.window.mvprintw(1, 0, border);
                for b in &self.buttons {
                    let (x, y) = b.find_grid_pos();
                    self.window.mvprintw(
                        y,
                        x,
                        format!("{}", self.assets.get(&b.asset_name).unwrap()),
                    );
                }
                self.window.refresh();
            },
            Some(e) => {
                self.window.mvprintw(1, 0, "Error");
                self.window.mvprintw(2, 0, e.to_string());
            }
        }
    }

    fn display_error(&mut self, error: Error) {
        self.error = Some(error);
    }
}
