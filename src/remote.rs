use anyhow::{Context, Result, bail};
use native_tls::{TlsConnector, TlsStream};
use std::{fs::File, io::{Read, Write}, net::TcpStream, path::{Path, PathBuf}};
use tungstenite::{Message, WebSocket, client};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct WsData {
    id: String,
    token: Option<String>
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
    Menu
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

pub struct Remote {
    socket: WebSocket<TlsStream<TcpStream>>,
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
        return cfg.join("b-controller.token.txt");
    }

    fn get_token() -> Option<String> {
        let path = Remote::get_token_path();
        if path.is_file() {
            let mut f = match File::open(path) {
                Ok(fp) => fp,
                _ => return None
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
        let name = "Yi1jb250cm9sbGVy";
        let mut url = format!(
            "wss://{}:{}/api/v2/channels/samsung.remote.control?name={}",
            hostname, port, name
        );
        let token = Remote::get_token();
        if token.is_some() {
            url.push_str("&token=");
            url.push_str(&token.unwrap());
        }

        let tcp = TcpStream::connect(format!("{}:{}", hostname, port))
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let connector = TlsConnector::builder()
            .danger_accept_invalid_hostnames(true)
            .danger_accept_invalid_certs(true)
            .build()
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let stream = connector.connect(&hostname, tcp)
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let (mut socket, response) = client(url, stream)
            .with_context(|| "Failed to connect to Samsung TV:")?;
        if response.status().is_client_error() || response.status().is_server_error() {
            let status = response.status();
            let str = String::from_utf8(response.into_body().unwrap_or_default()).unwrap_or_default();
            bail!("Failed to connect to TV: {} {}", status, str);
        }
        let msg = socket.read().with_context(|| "Failed to connect to Samsung TV:")?;
        let parsed: WsMessage = serde_json::from_str(msg.into_text()?.as_str())?;
        if parsed.data.token.is_some() {
            Remote::set_token(&parsed.data.token.unwrap())?;
        }
        return Ok(Remote { socket });
    }

    pub fn send_key(&mut self, key: &Key) -> Result<()> {
        let payload = format!(r#"{{
            "method": "ms.remote.control",
            "params": {{
                "Cmd": "Click",
                "DataOfCmd": "{}",
                "Option": "false",
                "TypeOfRemote": "SendRemoteKey"
            }}
        }}"#, key.to_string());
        self.socket.send(Message::Text(payload.into()))?;
        return Ok(());
    }
    
    pub fn ping(&mut self) -> Result<()> {
        self.socket.send(Message::Ping("".into()))?;
        let _ = self.socket.read()?;
        return Ok(());
    }
}

