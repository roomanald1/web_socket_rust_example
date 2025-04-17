pub mod udp;
mod websocket;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration};
use bytes::Bytes;
use native_tls::{TlsConnector, TlsStream};
use serde_json::json;
use tokio::time::Instant;
use tungstenite::{client, Message, WebSocket};
use tungstenite::http::Uri;

const OKX_WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut ws = create_connection()?;
    let mut last_ping = Instant::now();
    let ping_interval = Duration::from_secs(10);

    loop {
        if last_ping.elapsed() >= ping_interval && ws.can_write() {
            match ws.send(Message::Ping(Bytes::new())) {
                Ok(()) => last_ping = Instant::now(),
                Err(e) => println!("Ping error: {}", e),
            }
        }

        // Read messages with non-blocking handling
        match ws.read() {
            Ok(msg) => println!("Received: {}", msg),
            Err(tungstenite::Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));//tiny sleep
            }
            Err(tungstenite::Error::ConnectionClosed) => {
                eprintln!("Connection closed");
                ws = create_connection()?;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                ws = create_connection()?;
            }
        }
    }
}


fn create_connection() -> Result<WebSocket<TlsStream<TcpStream>>, Box<dyn std::error::Error>> {
    println!("Connecting to {}...", OKX_WS_URL);
    let uri: Uri = OKX_WS_URL.parse()?;
    let host = uri.host().ok_or("No host in URL")?;
    let port = uri.port_u16().unwrap_or(8443);

    let stream = TcpStream::connect((host, port))?;
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(host, stream)?;
    let (mut ws, _) = client(uri.clone(), tls_stream)?;
    ws.get_mut().get_mut().set_nonblocking(true)?;

    println!("Successfully connected to server");

    println!("Subscribing to BTC-USD-251226-85000-C");
    ws.send(Message::Text(json!({
        "op": "subscribe",
        "args": [
            {
                "channel": "tickers",
                "instFamily": "BTC-USD",
                "instId": "BTC-USD-251226-85000-C"
            }
        ]
    }).to_string().into())).map_err(|e| format!("Subscription send error: {}", e))?;
    Ok(ws)
}

