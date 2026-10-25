use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    vec,
};
use tungstenite::{Message, client};

use crate::http::{HttpClient, HttpResponse};

#[derive(Serialize, Deserialize)]
struct WsData {
    id: String,
    token: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct WsMessage {
    data: WsData,
    event: String,
}

pub enum Key {
    Power,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Play,
    Pause,
    Mute,
    VolUp,
    VolDown,
    ChUp,
    ChDown,
    ChList,
    Home,
    Menu,
}

impl Key {
    pub fn to_string(&self) -> String {
        let str = match self {
            Key::Power => "KEY_POWER",
            Key::Up => "KEY_UP",
            Key::Down => "KEY_DOWN",
            Key::Left => "KEY_LEFT",
            Key::Right => "KEY_RIGHT",
            Key::Enter => "KEY_ENTER",
            Key::Back => "KEY_RETURN",
            Key::Play => "KEY_PLAY",
            Key::Pause => "KEY_PAUSE",
            Key::Mute => "KEY_MUTE",
            Key::VolUp => "KEY_VOLUP",
            Key::VolDown => "KEY_VOLDOWN",
            Key::ChUp => "KEY_CHUP",
            Key::ChDown => "KEY_CHDOWN",
            Key::ChList => "KEY_CH_LIST",
            Key::Home => "KEY_HOME",
            Key::Menu => "KEY_MENU",
        };
        return str.to_owned();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum App {
    YouTube,
    Netflix,
    PrimeVideo,
    Disney,
    HBO,
    Spotify,
    AppleTV,
    Paramount,
    Peacock,
    Hulu,
    Twitch,
}

impl App {
    pub fn get_id(&self) -> &str {
        return match self {
            App::Netflix => "3201907018807",
            App::PrimeVideo => "3201910019365",
            App::Disney => "3202204027038",
            App::HBO => "3202301029760",
            App::Spotify => "3201606009684",
            App::YouTube => "111299001912",
            App::AppleTV => "3201807016597",
            App::Paramount => "3202110025305",
            App::Peacock => "3202006020991",
            App::Hulu => "3201601007625",
            App::Twitch => "3202203026841",
        };
    }
    pub fn list() -> Vec<App> {
        let apps = vec![
            App::YouTube,
            App::Netflix,
            App::PrimeVideo,
            App::Disney,
            App::HBO,
            App::Spotify,
            App::AppleTV,
            App::Paramount,
            App::Peacock,
            App::Hulu,
            App::Twitch,
        ];
        return apps;
    }
    pub fn name(&self) -> &str {
        return match self {
            App::Netflix => "netflix",
            App::PrimeVideo => "prime-video",
            App::Disney => "disney",
            App::HBO => "hbo",
            App::Spotify => "spotify",
            App::YouTube => "youtube",
            App::AppleTV => "apple-tv",
            App::Paramount => "paramount",
            App::Peacock => "peacock",
            App::Hulu => "hulu",
            App::Twitch => "twitch",
        };
    }
}

enum Command {
    WsMessage(Message),
    GetRequest(String),
    PostRequest(String),
}

pub struct Remote {
    exit: Arc<AtomicBool>,
    tx: Sender<Command>,
    rx: Receiver<HttpResponse>,
    handle: Option<JoinHandle<()>>,
}

impl Remote {
    fn get_token_path() -> PathBuf {
        let cfg = match std::env::var("XDG_CONFIG_HOME") {
            Ok(dir) => Path::new(&dir).to_owned(),
            _ => match std::env::var("HOME") {
                Ok(dir) => Path::new(&dir).join(".config"),
                _ => Path::new(".").to_owned(),
            },
        };
        return cfg.join("b-remote.token.txt");
    }

    fn get_token() -> Option<String> {
        let path = Remote::get_token_path();
        if path.is_file() {
            let mut f = match File::open(path) {
                Ok(fp) => fp,
                _ => return None,
            };
            let mut content = String::new();
            match f.read_to_string(&mut content) {
                Err(_) => return None,
                _ => (),
            }
            return Some(content.trim().to_owned());
        }
        return None;
    }

    fn set_token(token: &str) -> Result<()> {
        let path = Remote::get_token_path();
        let mut f = File::create(path)?;
        f.write_all(token.as_bytes())?;
        return Ok(());
    }

    pub fn new() -> Result<Remote> {
        let hostname = "Samsung.local";
        let port = 8002;
        let name = "Yi1yZW1vdGU=";
        let mut url = format!(
            "wss://{}:{}/api/v2/channels/samsung.remote.control?name={}",
            hostname, port, name
        );
        let token = Remote::get_token();
        if token.is_some() {
            url.push_str("&token=");
            url.push_str(&token.unwrap());
        }

        let mut http_client = HttpClient::new(&hostname, port)?;
        let stream = HttpClient::open_tls_stream(&hostname, port)?;
        let (mut socket, response) =
            client(url, stream).with_context(|| "Failed to connect to Samsung TV:")?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let status = response.status();
            let str =
                String::from_utf8(response.into_body().unwrap_or_default()).unwrap_or_default();
            bail!("Failed to connect to TV: {} {}", status, str);
        }
        let msg = socket
            .read()
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let parsed: WsMessage = serde_json::from_str(msg.into_text()?.as_str())?;
        if parsed.data.token.is_some() {
            Remote::set_token(&parsed.data.token.unwrap())?;
        }
        let exit = Arc::new(AtomicBool::new(false));
        let clone = exit.clone();
        let (tx, rx) = mpsc::channel();
        let (http_tx, http_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            while !clone.load(Ordering::Relaxed) {
                let cmd: Command = rx.recv().unwrap();
                match cmd {
                    Command::WsMessage(msg) => {
                        let is_ping = msg.is_ping();
                        let _ = socket.send(msg).unwrap();
                        if is_ping {
                            let _ = socket.read().unwrap();
                        }
                    }
                    Command::PostRequest(path) => {
                        let res = http_client.post(&path).unwrap();
                        http_tx.send(res).unwrap();
                    }
                    Command::GetRequest(path) => {
                        let res = http_client.get(&path).unwrap();
                        http_tx.send(res).unwrap();
                    }
                }
            }
        });
        return Ok(Remote {
            handle: Some(handle),
            exit,
            tx,
            rx: http_rx,
        });
    }

    pub fn send_key(&mut self, key: &Key) -> Result<()> {
        let payload = format!(
            r#"{{
            "method": "ms.remote.control",
            "params": {{
                "Cmd": "Click",
                "DataOfCmd": "{}",
                "Option": "false",
                "TypeOfRemote": "SendRemoteKey"
            }}
        }}"#,
            key.to_string()
        );
        self.tx
            .send(Command::WsMessage(Message::Text(payload.into())))?;
        return Ok(());
    }

    pub fn ping(&mut self) -> Result<()> {
        self.tx.send(Command::WsMessage(Message::Ping("".into())))?;
        return Ok(());
    }

    pub fn open_app(&mut self, app: App) -> Result<bool> {
        let url: String = format!("/api/v2/applications/{}", app.get_id()).into();
        self.tx.send(Command::PostRequest(url))?;
        let res = self.rx.recv()?;
        if res.status_code == 200 && res.body == "true" {
            return Ok(true);
        }
        return Ok(false);
    }

    pub fn get_apps(&mut self) -> Result<Vec<App>> {
        let mut apps = Vec::new();
        let list: Vec<App> = App::list();
        for app in list {
            let url: String = format!("/api/v2/applications/{}", app.get_id()).into();
            self.tx.send(Command::GetRequest(url))?;
            let res = self.rx.recv()?;
            if res.status_code == 200 {
                apps.push(app);
            }
        }
        return Ok(apps);
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Relaxed);
        let _ = self.ping();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
