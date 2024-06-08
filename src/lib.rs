use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;
use warp::Rejection;
use warp::Reply;

use crate::ws::Clients;
pub mod ws;

// pub const PORT: u32 = 8080;
type Result<T> = std::result::Result<T, Rejection>;
pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    //upgrade socket
    println!("Ws handler");
    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients.clone())))
}
pub async fn run() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    println!("Configuring websocket");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
