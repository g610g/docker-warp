use crate::docker::WsDocker;
use crate::ChannelReciever;
use futures::{SinkExt, StreamExt, TryFutureExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::WebSocket;
use warp::{filters::ws::Message, reject::Rejection};
#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<Message>>,
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Result<T> = std::result::Result<T, Rejection>;
type _Channel = (UnboundedSender<Message>, ChannelReciever<Message>);
pub async fn client_connection(ws: WebSocket, clients: Clients, _ws: WsDocker) {
    println!("Establishing client connection... {:?}", ws);
    println!("Initializing Docker interface...");
    let (mut ws_client_sender, _ws_client_receiver) = ws.split();
    //creates an unbounded channel

    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let mut client_rcv = UnboundedReceiverStream::new(client_rcv);

    //channel recieves async recieves messages and forwards to the ws sink
    tokio::spawn(async move {
        while let Some(message) = client_rcv.next().await {
            println!("recieves the message {:?} from channel reciever", message);
            let _ = ws_client_sender
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    let uuid = uuid::Uuid::new_v4().simple().to_string();
    //creates a new client
    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };
    clients.lock().await.insert(uuid.clone(), new_client);

    //create tasks that each will get the stream of data for each containers

    // clients.lock().await.remove(&uuid);
    // println!("user: {} disconnected", uuid);
}
async fn client_message(client_id: &str, msg: Message, clients: &Clients) {
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    let new_message = format!("Message from user {} : {}", client_id, message);
    let locked = clients.lock().await;
    for (_, val) in locked.iter() {
        if let Some(sender) = &val.sender {
            let _ = sender.send(Message::text(new_message.clone()));
        }
    }
}
