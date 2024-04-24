use std::{env, net::SocketAddr, time::Duration};
use std::fmt::format;

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


async fn mqtt_open(user : &str, pass: &str) -> (AsyncClient, EventLoop) {

    info!("Open Mqtt connection");

    // Mosquitto
    let uuid = Uuid::new_v4();
    let mut mqttoptions = MqttOptions::new(format!("bridge-client-{}", uuid), "192.168.0.149", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30_000));
    mqttoptions.set_clean_start(true);
    mqttoptions.set_credentials(user, pass);

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

///
/// host ,  mqtt-user , mqtt-pass
///
fn read_host() -> (String, String, String) {
    // Récupérer les arguments de la ligne de commande
    let args: Vec<String> = env::args().collect();

    // Variables pour stocker les valeurs extraites
    let mut host_address: Option<String> = None;
    let mut mqtt_user: Option<String> = None;
    let mut mqtt_pass: Option<String> = None;

    // Parcourir les arguments pour extraire les valeurs
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--host" if i + 1 < args.len() => {
                host_address = Some(args[i + 1].clone());
                i += 2; // Avancer de deux pas pour sauter l'argument et la valeur
            }
            "--mqtt-user" if i + 1 < args.len() => {
                mqtt_user = Some(args[i + 1].clone());
                i += 2; // Avancer de deux pas pour sauter l'argument et la valeur
            }
            "--mqtt-pass" if i + 1 < args.len() => {
                mqtt_pass = Some(args[i + 1].clone());
                i += 2; // Avancer de deux pas pour sauter l'argument et la valeur
            }
            _ => {
                i += 1; // Avancer d'un pas
            }
        }
    }

    // Afficher les valeurs extraites
    let ip = if let Some(ip) = host_address {
        println!("Adresse IP de l'hôte : {}", ip);
        ip
    } else { panic!("Missing host address")};
    let user = if let Some(user) = mqtt_user {
        println!("Nom d'utilisateur MQTT : {}", user);
        user
    } else { panic!("Missing mqtt user")};
    let pass = if let Some(pass) = mqtt_pass {
        println!("Mot de passe MQTT : ...");
        pass
    } else { panic!("Missing mqtt password")};

    (ip, user, pass)
}


#[tokio::main]
async fn main() {

    // --host 192.168.0.99

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting Mqtt Bridge 0.7.0");

    let (ip, mqtt_user, mqtt_pass) = read_host();

    // ** Web Socket **
    let addr = format!("{}:9002", ip);// "192.168.0.99:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((mut stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (ws_sender, ws_receiver) = ws_stream.split();

        let (mut client, mut eventloop) = mqtt_open(&mqtt_user, &mqtt_pass).await;
        tokio::spawn(accept_connection_for_reading_ws(peer, ws_receiver, client));
        tokio::spawn(accept_connection_for_writing_ws(peer, ws_sender, eventloop));
    }
}
