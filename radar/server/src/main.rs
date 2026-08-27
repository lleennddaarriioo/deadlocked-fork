use std::{net::SocketAddr, time::Duration};

use axum::{
    Router,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use serde::{Serialize, de::DeserializeOwned};
use shared::data::Data;
use tokio::{net::TcpListener, time::sleep};
use tower_http::services::ServeDir;
use utils::log::LoggerOptions;

use crate::game::Games;

mod game;

#[tokio::main]
async fn main() {
    utils::log::init(LoggerOptions::default(), |w, rec| {
        writeln!(w, "[{}] {}", rec.level, rec.args)
    })
    .unwrap();

    let assets_dir = std::env::current_dir().unwrap().join("assets");
    let router = Router::new()
        .fallback_service(ServeDir::new(assets_dir))
        .route("/server", any(server_handler))
        .route("/client", any(client_handler))
        .with_state(Games::default());

    let listener = TcpListener::bind("127.0.0.1:6346").await.unwrap();
    utils::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn server_handler(
    State(state): State<Games>,
    ws: WebSocketUpgrade,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| server(state, ws))
}

async fn server(state: Games, mut ws: WebSocket) {
    while let Some(data) = recv_json::<Data>(&mut ws).await {
        let mut game_data = state.write().await;
        *game_data = data;
    }
}

async fn client_handler(
    State(state): State<Games>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut ip = addr.ip().to_string();
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            ip = forwarded_str.to_string();
        }
    }

    let ip_clone = ip.clone();
    
    // Ignore local IPs and the host's public IP
    let is_ignored = ip_clone == "127.0.0.1" 
        || ip_clone == "::1" 
        || ip_clone.starts_with("192.168.") 
        || ip_clone.starts_with("10.") 
        || ip_clone == "45.172.117.61";

    if !is_ignored {
        tokio::spawn(async move {
            if let Ok(res) = reqwest::get(format!("http://ip-api.com/json/{}", ip_clone)).await {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let (Some(country), Some(city), Some(isp)) = (
                        json.get("country").and_then(|v| v.as_str()),
                        json.get("city").and_then(|v| v.as_str()),
                        json.get("isp").and_then(|v| v.as_str()),
                    ) {
                        utils::info!("[IP GRAB] Incoming connection from: {} | Location: {}, {} | ISP: {}", ip_clone, city, country, isp);
                        return;
                    }
                }
            }
            utils::info!("[IP GRAB] Incoming connection from public IP: {} (Location lookup failed)", ip_clone);
        });
    }

    ws.on_upgrade(|ws| client(state, ws))
}

async fn client(state: Games, mut ws: WebSocket) {
    loop {
        let data = state.read().await.clone();
        if send_json(&mut ws, &data).await.is_none() {
            return;
        }
        sleep(Duration::from_millis(8)).await;
    }
}

async fn send_json<T: Serialize>(ws: &mut WebSocket, value: &T) -> Option<()> {
    let Ok(json) = serde_json::to_string(value) else {
        return None;
    };

    ws.send(Message::Text(json.into())).await.ok()
}

async fn recv_json<T: DeserializeOwned>(ws: &mut WebSocket) -> Option<T> {
    let Some(Ok(msg)) = ws.recv().await else {
        return None;
    };

    let Ok(text) = msg.into_text() else {
        return None;
    };

    serde_json::from_str(&text).ok()
}
