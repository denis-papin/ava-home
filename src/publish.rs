// #[macro_use]
// extern crate log;

use std::env;
use std::io::{Write};
use std::net::TcpStream;


use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::{Decodable, Encodable, QualityOfService};
use mqtt::{TopicFilter, TopicName};
use crate::Params;

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

pub fn connect_publisher(server_addr: &str) -> TcpStream {
    info!("Connect the publisher");

    let client_id = generate_client_id();
    let keep_alive = 30_000;

    info!("Connecting to {:?} ... ", server_addr);
    let mut stream = TcpStream::connect(server_addr).unwrap();
    info!("Connected!");

    info!("Client identifier {:?}", client_id);
    let mut conn = ConnectPacket::new(client_id);
    conn.set_clean_session(true);
    conn.set_keep_alive(keep_alive);
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();

    stream.write_all(&buf[..]).unwrap();

    let connack = ConnackPacket::decode(&mut stream).unwrap();
    trace!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    stream
}

pub fn publish(/*mut*/ stream : &mut TcpStream, topic: &str, message: &str ) {

    let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![(TopicFilter::new(topic).unwrap(), QualityOfService::Level0)];

    info!("Applying channel filters {:?} ...", channel_filters);
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let channels: Vec<TopicName> = vec![TopicName::new(topic).unwrap()];

    println!("Message : {}", message);

    for chan in &channels {
        let publish_packet = PublishPacketRef::new(chan, QoSWithPacketIdentifier::Level0, message.as_bytes());
        let mut buf = Vec::new();
        publish_packet.encode(&mut buf).unwrap();
        stream.write_all(&buf[..]).unwrap();
    }

}


