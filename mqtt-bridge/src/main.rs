use std::{env, net::SocketAddr, time::Duration};

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use log::*;
use rumqttc::v5::{AsyncClient, Event, EventLoop, Incoming, MqttOptions};
use rumqttc::v5::mqttbytes::QoS;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::{Error, Message, Result}, WebSocketStream};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

async fn accept_connection_for_writing_ws(peer: SocketAddr, mut stream: SplitSink<WebSocketStream<TcpStream>, Message>, mut eventloop: EventLoop) {
    if let Err(e) = handle_writing_ws(peer, stream, eventloop).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {:?}", err),
        }
    }
}

async fn accept_connection_for_reading_ws(peer: SocketAddr, mut ws_receiver: SplitStream<WebSocketStream<TcpStream>>, mut client: AsyncClient) {
    if let Err(e) = handle_reading_ws(peer, ws_receiver, client).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {:?}", err),
        }
    }
}

///
/// Handle incoming WS messages
async fn handle_reading_ws(peer: SocketAddr, mut ws_receiver:  SplitStream<WebSocketStream<TcpStream>>, mut client: AsyncClient) -> Result<()> {
    println!("Send a message");
    loop {
        println!("Loop is spinning");
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            info!("WS message received : {}", msg);
                            let bridge_message : BridgeMessage = serde_json::from_str(&msg.to_string()).unwrap(); // TODO
                            info!("📨 Client [{}] sent message to mqtt, topic=[{}], message=[{}]", &peer, &bridge_message.topic,  &bridge_message.raw_message);
                            let data = bridge_message.raw_message.as_bytes().to_vec();
                            client.publish(&bridge_message.topic, QoS::AtLeastOnce, false, data).await.unwrap(); // TODO
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct BridgeMessage {
    topic: String,
    raw_message: String, // Json string of the actual message
}


async fn mqtt_open() -> (AsyncClient, EventLoop) {

    info!("Open Mqtt connection");

    // Mosquitto
    let uuid = Uuid::new_v4();
    let mut mqttoptions = MqttOptions::new(format!("bridge-client-{}", uuid), "192.168.0.149", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30_000));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials("ava", "avatece3.X");

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    info!("Subscribe to channels");
    client.subscribe("*", QoS::AtMostOnce).await.unwrap();
    client.subscribe("zigbee2mqtt/ts_salon_1", QoS::AtMostOnce).await.unwrap();
    client.subscribe("zigbee2mqtt/bureau", QoS::AtMostOnce).await.unwrap();
    client.subscribe("zigbee2mqtt/ts_chambre_1", QoS::AtMostOnce).await.unwrap();
    client.subscribe("zigbee2mqtt/ts_couloir", QoS::AtMostOnce).await.unwrap();

    client.subscribe("regulator/regulate_radiator", QoS::AtMostOnce).await.unwrap();

    client.subscribe("external/rad_salon", QoS::AtMostOnce).await.unwrap();
    client.subscribe("external/rad_bureau", QoS::AtMostOnce).await.unwrap();
    client.subscribe("external/rad_chambre", QoS::AtMostOnce).await.unwrap();
    client.subscribe("external/rad_couloir", QoS::AtMostOnce).await.unwrap();

    (client, eventloop)
}

async fn handle_writing_ws(peer: SocketAddr, mut ws_sender: SplitSink<WebSocketStream<TcpStream>, Message>, mut eventloop: EventLoop) -> Result<()> {
    info!("WebSocket connection: {}", &peer);

    while let Ok(notification) = eventloop.poll().await {
        // info!("Enter notification pool");
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                let topic = std::str::from_utf8(publish.topic.as_ref()).unwrap(); // TODO
                let msg = std::str::from_utf8(&publish.payload).unwrap();
                info!("🧶 Client [{}] is reading mqtt message on topic: [{}], message: <{}>", &peer, topic, msg);

                // Send the mqtt message to the ws client
                println!("Send a WS message to the web client: [{}]", &peer);
                let m = serde_json::to_string(&BridgeMessage {
                    topic: topic.to_string(),
                    raw_message: msg.to_string(),
                }).unwrap();
                let m = Message::Text(m);
                ws_sender.send(m).await?;

            }
            Event::Incoming(Incoming::ConnAck(_connack)) => {
            }
            Event::Incoming(Incoming::PubAck(_pub_ack)) => {
            }
            _ => {}
        }
    }
    info!("Exit client [{}] notification loop", &peer);
    Ok(())
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting Mqtt Bridge 0.7.0");

    // ** Web Socket **
    // let addr = "127.0.0.1:9002";
    let addr = "192.168.0.99:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((mut stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (ws_sender, ws_receiver) = ws_stream.split();

        let (mut client, mut eventloop) = mqtt_open().await;
        tokio::spawn(accept_connection_for_reading_ws(peer, ws_receiver, client));
        tokio::spawn(accept_connection_for_writing_ws(peer, ws_sender, eventloop));
    }
}




// //! A chat server that broadcasts a message to all connections.
// //!
// //! This is a simple line-based server which accepts WebSocket connections,
// //! reads lines from those connections, and broadcasts the lines to all other
// //! connected clients.
// //!
// //! You can test this out by running:
// //!
// //!     cargo run --example server 127.0.0.1:12345
// //!
// //! And then in another window run:
// //!
// //!     cargo run --example client ws://127.0.0.1:12345/
// //!
// //! You can run the second command in multiple windows and then chat between the
// //! two, seeing the messages from the other client as they're received. For all
// //! connected clients they'll all join the same room and see everyone else's
// //! messages.
//
// use std::{
//     collections::HashMap,
//     env,
//     io::Error as IoError,
//     net::SocketAddr,
//     sync::{Arc, Mutex},
// };
//
// use futures_channel::mpsc::{unbounded, UnboundedSender};
// use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
//
// use tokio::net::{TcpListener, TcpStream};
// use tokio_tungstenite::tungstenite::protocol::Message;
//
// type Tx = UnboundedSender<Message>;
// type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
//
// async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
//     println!("Incoming TCP connection from: {}", addr);
//
//     let ws_stream = tokio_tungstenite::accept_async(raw_stream)
//         .await
//         .expect("Error during the websocket handshake occurred");
//     println!("WebSocket connection established: {}", addr);
//
//     // Insert the write part of this peer to the peer map.
//     let (tx, rx) = unbounded();
//     peer_map.lock().unwrap().insert(addr, tx);
//
//     let (outgoing, incoming) = ws_stream.split();
//
//     let broadcast_incoming = incoming.try_for_each(|msg| {
//         println!("Received a message from {}: {}", addr, msg.to_text().unwrap());
//         let peers = peer_map.lock().unwrap();
//
//         // We want to broadcast the message to everyone except ourselves.
//         let broadcast_recipients =
//             peers.iter().filter(|(peer_addr, _)| peer_addr != &&addr).map(|(_, ws_sink)| ws_sink);
//
//         for recp in broadcast_recipients {
//             recp.unbounded_send(msg.clone()).unwrap();
//         }
//
//         future::ok(())
//     });
//
//     let receive_from_others = rx.map(Ok).forward(outgoing);
//
//     pin_mut!(broadcast_incoming, receive_from_others);
//     future::select(broadcast_incoming, receive_from_others).await;
//
//     println!("{} disconnected", &addr);
//     peer_map.lock().unwrap().remove(&addr);
// }
//
// #[tokio::main]
// async fn main() -> Result<(), IoError> {
//     let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
//
//     let state = PeerMap::new(Mutex::new(HashMap::new()));
//
//     // Create the event loop and TCP listener we'll accept connections on.
//     let try_socket = TcpListener::bind(&addr).await;
//     let listener = try_socket.expect("Failed to bind");
//     println!("Listening on: {}", addr);
//
//     // Let's spawn the handling of each connection in a separate task.
//     while let Ok((stream, addr)) = listener.accept().await {
//         tokio::spawn(handle_connection(state.clone(), stream, addr));
//     }
//
//     Ok(())
// }
