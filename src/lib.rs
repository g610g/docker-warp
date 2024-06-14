use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use crate::ws::Clients;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use warp::filters::ws::Message;
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
    clients: Clients,
    channel: (UnboundedSender<Message>, ChannelReciever<Message>),
) -> Result<impl Reply> {
    //upgrade socket
    println!("Ws handler");
    let (rt, rx) = channel;
    Ok(ws.on_upgrade(move |socket| {
        ws::client_connection(socket, clients.clone(), (rt.clone(), rx.clone()))
    }))
}
pub async fn run() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let (rt, rx) = mpsc::unbounded_channel::<Message>();
    println!("Configuring websocket");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_channel((rt, Arc::new(rx))))
        .and_then(ws_handler);
    let routes = ws_route.with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
fn with_channel(
    channel: (UnboundedSender<Message>, ChannelReciever<Message>),
) -> impl Filter<Extract = ((UnboundedSender<Message>, ChannelReciever<Message>),), Error = Infallible>
       + Clone {
    let (rt, rx) = channel;
    warp::any().map(move || (rt.clone(), rx.clone()))
}
fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
