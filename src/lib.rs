use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use crate::ws::Clients;
use docker_api::Docker;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use warp::Filter;
use warp::Rejection;
use warp::Reply;
pub mod docker;
pub mod warp_example;
pub mod ws;
// pub const PORT: u32 = 8080;
type Result<T> = std::result::Result<T, Rejection>;
pub type ChannelReciever<T> = Arc<UnboundedReceiver<T>>;
pub async fn ws_handler(
    ws: warp::ws::Ws,
    clients_and_docker: (Clients, Docker),
) -> Result<impl Reply> {
    //upgrade socket
    println!("Ws handler");
    let (clients, docker) = clients_and_docker;

    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients.clone(), docker)))
}
pub async fn run() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let docker = docker_api::Docker::unix("/var/run/docker.sock");
    println!("Configuring websocket");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone(), docker.clone()))
        .and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
// fn with_channel(
//     channel: (UnboundedSender<Message>, ChannelReciever<Message>),
// ) -> impl Filter<Extract = ((UnboundedSender<Message>, ChannelReciever<Message>),), Error = Infallible>
//        + Clone {
//     let (rt, rx) = channel;
//     warp::any().map(move || (rt.clone(), rx.clone()))
// }
fn with_clients(
    clients: Clients,
    docker_instance: Docker,
) -> impl Filter<Extract = ((Clients, Docker),), Error = Infallible> + Clone {
    warp::any().map(move || (clients.clone(), docker_instance.clone()))
}
