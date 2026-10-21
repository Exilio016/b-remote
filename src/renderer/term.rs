use anyhow::Error;
use crosscurses::{ALL_MOUSE_EVENTS, Input, MEVENT, REPORT_MOUSE_POSITION, Window, initscr};
use std::{ptr::null_mut};

use crate::{remote::Remote, renderer::{Asset, KeyCode, Position, Renderer}};

use super::Button;

pub struct TermRenderer {
    buttons: [Button; 17],
    running: bool,
    window: Window,
    error: Option<Error>,
    has_drawn_error: bool,
}

const GRID_ROWS: f32 = 9.0;
const GRID_COLS: f32 = 7.0;
const W_WIDTH: f32 = 15.0;
const W_HEIGHT: f32 = 8.0;

impl Position {
    fn find_grid_pos(&self) -> (i32, i32) {
        let cell_width = W_WIDTH / GRID_COLS;
        let cell_height = W_HEIGHT / GRID_ROWS;
        return (
            2 + (self.x as f32 * cell_width) as i32,
            3 + (self.y as f32 * cell_height) as i32,
        );
    }
}

impl KeyCode {
    pub fn to_crossterm(&self) -> Input {
        match self {
            KeyCode::Equal => Input::Character('='),
            KeyCode::Minus => Input::Character('-'),
            KeyCode::Up => Input::KeyUp,
            KeyCode::Down => Input::KeyDown,
            KeyCode::Left => Input::KeyLeft,
            KeyCode::Right => Input::KeyRight,
            KeyCode::Space => Input::Character(' '),
            KeyCode::Escape => Input::Character('\u{1b}'),
            KeyCode::Enter => Input::Character('\n'),
            KeyCode::M => Input::Character('M'),
            KeyCode::P => Input::Character('P'),
            KeyCode::H => Input::Character('H'),
            KeyCode::J => Input::Character('J'),
            KeyCode::K => Input::Character('K'),
            KeyCode::Z => Input::Character('Z'),
            KeyCode::S => Input::Character('S'),
            KeyCode::L => Input::Character('L'),
        }
    }
}

impl Asset {
    pub fn get_char(&self) -> char {
        match self.name() {
            "circle-power" => '⏻',
            "arrow-up" => '▲',
            "volume-x" => '🔇',
            "arrow-down" => '▼',
            "tv" => '📺',
            "circle-arrow-up" => '▲',
            "circle-arrow-left" => '◄',
            "circle" => '●',
            "circle-arrow-right" => '►',
            "circle-arrow-down" => '▼',
            "undo-2" => '↩',
            "play" => '▶',
            "pause" => '⏸',
            "house" => '⌂',
            "settings" => '⚙',
            _ => '?'
        }
    }
}

impl Renderer for TermRenderer {
    fn new(buttons: [Button; 17]) -> Self {
        let window = initscr();
        crosscurses::curs_set(0);
        window.keypad(true);
        crosscurses::mousemask(ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION, null_mut());
        let mut term = TermRenderer {
            buttons,
            running: true,
            window,
            error: None,
            has_drawn_error: false,
        };
        term.draw();
        return term;
    }

    fn is_running(&self) -> bool {
        return self.running;
    }

    fn update(&mut self, remote: &mut Option<Remote>) {
        if self.error.is_some() && !self.has_drawn_error {
            return;
        }
        let keycode = self.get_keycode();
        let mut mevent: Option<MEVENT> = None;
        if keycode == Input::KeyMouse {
            mevent = Some(crosscurses::getmouse().unwrap());
        }
        else if keycode == Input::Character('Q') {
            self.running = false;
        }

        if self.error.is_none() {
            match remote {
                Some(remote) => {
                    for b in &self.buttons {
                        let (x, y) = b.pos.find_grid_pos();
                        if keycode == b.shortcut.to_crossterm()
                            || (mevent.is_some()
                                && x == mevent.unwrap().x
                                && y == mevent.unwrap().y)
                        {
                            let _ = remote.send_key(&b.cmd);
                        }
                    }
                    let _ = remote.ping();
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
        self.window.clear();

        match &self.error {
            None => {
                // Draw title
                self.window.mvprintw(0, 0, "b-remote");
                self.window.mvprintw(0, 13, "Press q to quit");

                // Draw buttons
                for b in &self.buttons {
                    let (x, y) = b.pos.find_grid_pos();
                    let ch = b.asset.get_char();
                    self.window.mvprintw(y, x, &format!("{}", ch));
                }

                // Draw shortcut legend at bottom
                let legend_y = 13;
                self.window.mvprintw(
                    legend_y,
                    0,
                    "P=Power  +=VolUp  M=Mute  -=VolDown  K=ChUp  L=ChList  J=ChDown",
                );
                self.window.mvprintw(
                    legend_y + 1,
                    0,
                    "Arrows=Nav  Enter=OK  Esc=Back  Space=Play  Z=Pause  H=Home  S=Menu",
                );
                self.window
                    .mvprintw(legend_y + 2, 0, "Mouse clicks work  |  q/Q to quit");

                self.window.refresh();
            }
            Some(e) => {
                self.window.mvprintw(0, 0, "b-remote - Error");
                let error_msg = e
                    .chain()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                for (i, line) in error_msg.lines().enumerate() {
                    self.window.mvprintw(2 + i as i32, 0, line);
                }
                self.window.mvprintw(12, 0, "Press q to quit");
                self.has_drawn_error = true;
                self.window.refresh();
            }
        }
    }

    fn display_error(&mut self, error: Error) {
        self.error = Some(error);
    }
}

impl TermRenderer {
    fn get_keycode(&self) -> Input {
        let key = self.window.getch();
        let keycode = match key {
            Some(input) => {
                match input {
                    Input::Character(c) => {
                        if c == '_' {
                            Input::Character('-')
                        }
                        else if c == '+' {
                            Input::Character('=')
                        }
                        else {
                            Input::Character(c.to_ascii_uppercase())
                        }
                    },
                    _ => input,
                }
            },
            None => Input::KeyCancel,
        };
        return keycode;
    }
}

