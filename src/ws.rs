use futures::{FutureExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::WebSocket;
use warp::{filters::ws::Message, reject::Rejection};
#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type Result<T> = std::result::Result<T, Rejection>;

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    println!("Establishing client connection... {:?}", ws);

    //splits the websocket into sender and reciever handler
    let (ws_client_sender, mut ws_client_receiver) = ws.split();
    //creates an unbounded channel
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    //reading the stream asynchronously
    tokio::spawn(client_rcv.forward(ws_client_sender).map(|result| {
        if let Err(e) = result {
            println!("Error sending websocket msg:{}", e);
        }
    }));

    let uuid = uuid::Uuid::new_v4().simple().to_string();

    //creates a new client
    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };
    clients.lock().await.insert(uuid.clone(), new_client);
    while let Some(result) = ws_client_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error recieving message for id {}: {}", uuid.clone(), e);
                break;
            }
        };
        client_message(&uuid, msg, &clients).await;
    }
    clients.lock().await.remove(&uuid);
    println!("{} disconnected", uuid);
}
async fn client_message(client_id: &str, msg: Message, clients: &Clients) {
    println!("recieved message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    if message == "ping" || message == "pong\n" {
        let locked = clients.lock().await;
        match locked.get(client_id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    println!("Sending pong");
                    let _ = sender.send(Ok(Message::text("pong")));
                }
            }
            None => return,
        }
        return;
    }
}
