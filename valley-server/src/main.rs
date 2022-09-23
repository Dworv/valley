use futures::{StreamExt, SinkExt, stream::SplitSink};
use tokio::{
    net::{
        TcpListener,
        TcpStream
    },
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedSender},
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

#[derive(Debug)]
enum QueueItem {
    Sender(SplitSink<WebSocketStream<TcpStream>, Message>),
    Msg(Message),
}

#[allow(unused)]
#[tokio::main]
async fn main() {
    const ADDRESS: &'static str = "localhost:6969";

    let mut listener = TcpListener::bind(ADDRESS).await.unwrap();
    println!("Listening to {}", ADDRESS);

    let (mut tx, mut rx) = unbounded_channel::<QueueItem>();

    let send_handle = spawn(async move {
        println!("Sender task started");
        let mut senders: Vec<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>> = Vec::new();
        use QueueItem::*;
        loop {
            match rx.recv().await {
                Some(item) => match item {
                    Sender(sender) => senders.push(sender),
                    Msg(msg) => {
                        println!("Sending message to all: {}", msg);
                        for sender in senders.iter_mut() {
                            println!("Sending message to a client.");
                            sender.send(msg.clone()).await;
                        }
                    }
                },
                None => break,
            }
        }
        println!("Sending task closed successfully.")
    });

    while let Ok((socket, _)) = listener.accept().await {
        let sender = tx.clone();
        tokio::spawn( async move {
            handle_ws(socket, sender).await;
        });
    }

    send_handle.await;
    println!("Sender task finished");
}

async fn handle_ws(stream: TcpStream, tx: UnboundedSender<QueueItem>) {
    println!("Accepting stream: {}", stream.peer_addr().unwrap());
    if let Ok(ws_stream) = accept_async(stream).await {
        let (sender, reciever) = ws_stream.split();
        if let Err(_) = tx.send(QueueItem::Sender(sender)) {
            return
        }
        reciever.for_each( |maybe_msg| {
            match maybe_msg {
                Ok(msg) => {
                    println!("Recieved message {}", msg);
                    tx.send(QueueItem::Msg(msg)).unwrap();
                },
                Err(_) => println!("Stream disconected."),
            } 
            async move {}
        }).await;
    }
}