use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use ping::ping;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub ip: String,
    pub ports: Vec<u16>,
}

#[derive(Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
    pub ports: Vec<u16>,
    pub last_checked: Option<Instant>,
    pub is_online: bool,
    pub open_ports: Vec<u16>,
}

impl Server {
    pub fn new(name: &str, ip: &str, mut ports: Vec<u16>) -> Self {
        ports.sort_unstable();
        Self {
            name: name.to_string(),
            ip: ip.to_string(),
            ports,
            last_checked: None,
            is_online: false,
            open_ports: vec![],
        }
    }
}

pub fn check_server_status(server: &mut Server, rt: &Runtime) {
    let ip_addr = match IpAddr::from_str(&server.ip) {
        Ok(ip) => ip,
        Err(_) => return,
    };

    server.is_online = ping(ip_addr, None, None, None, None, None).is_ok();
    server.last_checked = Some(Instant::now());

    let mut open_ports = vec![];
    for &port in &server.ports {
        let addr = SocketAddr::new(ip_addr, port);
        let is_open = rt.block_on(async {
            tokio::time::timeout(std::time::Duration::from_millis(500), TcpStream::connect(addr))
                .await
                .is_ok()
        });
        if is_open {
            open_ports.push(port);
        }
    }
    server.open_ports = open_ports;
}

