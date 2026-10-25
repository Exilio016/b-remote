use std::{io::{Read, Write}, net::TcpStream};

use anyhow::{Context, Result, bail};
use native_tls::{TlsConnector, TlsStream};

pub struct HttpClient {
    host: String,
    port: i16,
}

enum HttpMethod {
    GET,
    POST
}

impl HttpMethod {
    pub fn to_string(&self) -> String {
        return match self {
            HttpMethod::GET => "GET".into(),
            HttpMethod::POST => "POST".into(),
        }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: i32,
    pub body: String,
}

impl HttpResponse {
    pub fn from_string(payload: String) -> Result<Self> {
        let mut body = String::new();
        let mut is_body = false;
        let mut code: i32 = 0;
        for line in payload.lines() {
            if is_body {
                if body.len() > 0 {
                    body.push('\n');
                }
                body.push_str(line);
            }
            else if line.starts_with("HTTP/") {
                let comp: Vec<&str> = line.split(' ').collect();
                if comp.len() < 3 {
                    dbg!(line);
                    bail!("Failed to parse http control line: {}", line);
                }
                let status = comp.get(1).unwrap();
                code = status.parse()?;
            }
            else if line == "" {
                is_body = true;
            }
        }
        if code == 0 {
            bail!("Failed to parse http response: {}", payload);
        }
        return Ok(HttpResponse { status_code: code, body });
    }
}

impl HttpClient {
    pub fn open_tls_stream(hostname: &str, port: i16) -> Result<TlsStream<TcpStream>> {
        let tcp = TcpStream::connect(format!("{}:{}", hostname, port))
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let connector = TlsConnector::builder()
            .danger_accept_invalid_hostnames(true)
            .danger_accept_invalid_certs(true)
            .build()
            .with_context(|| "Failed to connect to Samsung TV:")?;
        let stream = connector.connect(&hostname, tcp)
            .with_context(|| "Failed to connect to Samsung TV:")?;
        return Ok(stream);
    }

    pub fn new(hostname: &str, port: i16) -> Result<Self> {
        let mut host = String::new();
        host.push_str(hostname);
        Ok(HttpClient { host, port })
    }

    fn request(&mut self, method: HttpMethod, path: &str) -> Result<HttpResponse> {
        let mut stream = HttpClient::open_tls_stream(self.host.as_str(), self.port)?;
        let payload = format!("{} {}\r\nHost: {}\r\nConnection: close\r\n\r\n", method.to_string(), path, &self.host);
        stream.write_all(payload.as_bytes())?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let res = HttpResponse::from_string(response)?;
        return Ok(res);
    }

    pub fn get(&mut self, path: &str) -> Result<HttpResponse> {
        return self.request(HttpMethod::GET, path);
    }
    pub fn post(&mut self, path: &str) -> Result<HttpResponse> {
        return self.request(HttpMethod::POST, path);
    }
}
