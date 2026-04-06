use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use log::{info, warn};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerStatus {
    pub online: bool,
    pub players_online: u32,
    pub players_max: u32,
    pub player_names: Vec<String>,
    pub version: String,
    pub motd: String,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            online: false,
            players_online: 0,
            players_max: 0,
            player_names: vec![],
            version: String::new(),
            motd: String::new(),
        }
    }
}

/// Ping a Minecraft server using the Server List Ping (SLP) protocol.
/// Returns a default offline status if the server is unreachable or times out.
pub async fn get_server_status(host: &str, port: u16) -> ServerStatus {
    match tokio::time::timeout(Duration::from_secs(4), ping(host, port)).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            warn!("Server ping failed for {}:{} — {}", host, port, e);
            ServerStatus::default()
        }
        Err(_) => {
            warn!("Server ping timed out for {}:{}", host, port);
            ServerStatus::default()
        }
    }
}

// ---------------------------------------------------------------------------
// SLP implementation (Minecraft 1.7+ status protocol)
// ---------------------------------------------------------------------------

async fn ping(host: &str, port: u16) -> Result<ServerStatus, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port)).await?;

    // ---- Build handshake + status request in one write ----
    let mut packet = Vec::new();

    // Handshake body: protocol_version=0, server_addr, port, next_state=1
    let mut body = Vec::new();
    body.extend(varint(0));           // protocol version
    body.extend(mc_string(host));     // server address
    body.extend_from_slice(&port.to_be_bytes()); // port (big-endian u16)
    body.extend(varint(1));           // next state: status

    // Handshake packet
    packet.extend(varint((body.len() + 1) as u32)); // length = id byte + body
    packet.push(0x00);                               // packet id
    packet.extend(body);

    // Status request packet (just an id byte)
    packet.extend(varint(1));
    packet.push(0x00);

    stream.write_all(&packet).await?;

    // ---- Read status response ----
    let _pkt_len = read_varint(&mut stream).await?;
    let _pkt_id  = read_varint(&mut stream).await?;
    let json_len = read_varint(&mut stream).await? as usize;

    let mut json_buf = vec![0u8; json_len];
    stream.read_exact(&mut json_buf).await?;

    let json: serde_json::Value = serde_json::from_slice(&json_buf)?;

    let players_online = json["players"]["online"].as_u64().unwrap_or(0) as u32;
    let players_max    = json["players"]["max"].as_u64().unwrap_or(0) as u32;
    let version        = json["version"]["name"].as_str().unwrap_or("").to_string();

    let motd = parse_description(&json["description"]);

    let player_names: Vec<String> = json["players"]["sample"]
        .as_array()
        .map(|arr| {
            arr.iter()
               .filter_map(|p| p["name"].as_str().map(String::from))
               .collect()
        })
        .unwrap_or_default();

    info!("Server {}:{} — {} / {} online, version {}", host, port, players_online, players_max, version);

    Ok(ServerStatus {
        online: true,
        players_online,
        players_max,
        player_names,
        version,
        motd,
    })
}

/// Parse Minecraft's "description" field which can be a plain string
/// or a JSON text component like `{"text": "..."}`.
fn parse_description(val: &serde_json::Value) -> String {
    let raw = match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => obj
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    };
    strip_format_codes(&raw)
}

/// Strip Minecraft formatting codes (§X) from a string.
fn strip_format_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' {
            chars.next(); // skip the code character
        } else {
            result.push(c);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// VarInt helpers
// ---------------------------------------------------------------------------

fn varint(mut value: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 { byte |= 0x80; }
        buf.push(byte);
        if value == 0 { break; }
    }
    buf
}

fn mc_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut buf = varint(bytes.len() as u32);
    buf.extend_from_slice(bytes);
    buf
}

async fn read_varint(stream: &mut TcpStream) -> std::io::Result<u32> {
    let mut result = 0u32;
    let mut shift  = 0u32;
    loop {
        let byte = stream.read_u8().await?;
        result |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 { break; }
        shift += 7;
        if shift >= 35 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "VarInt overflow",
            ));
        }
    }
    Ok(result)
}
