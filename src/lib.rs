use crate::ws::Clients;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use warp::Filter;
use warp::Rejection;
use warp::Reply;
pub mod bollard_docker;
pub mod docker;
pub mod warp_example;
pub mod ws;
// pub const PORT: u32 = 8080;
type Result<T> = std::result::Result<T, Rejection>;
pub type ChannelReciever<T> = Arc<UnboundedReceiver<T>>;
pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    //upgrade socket
    println!("Ws handler");
    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients)))
}
pub async fn run() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    //runs the docker client interface and already listening for the stream of data for each
    //containers
    let _ = bollard_docker::initialize(clients.clone())
        .await
        .unwrap_or_else(|e| eprintln!("docker module error: {e}"));
    println!("Configuring websocket");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients))
        .and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
// fn with_channel(
//     channel: (UnboundedSender<Message>, ChannelReciever<Message>),
// ) -> impl Filter<Extract = ((UnboundedSender<Message>, ChannelReciever<Message>),), Error = Infallible>
//        + Clone {
//     let (rt, rx) = channel;
//     warp::any().map(move || (rt.clone(), rx.clone()))
// }
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
