// #[macro_use]
// extern crate log;

use std::env;
use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;

use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::{Decodable, Encodable, QualityOfService};
use mqtt::{TopicFilter, TopicName};

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

pub fn connect_publisher() -> ( TcpStream, ConnectPacket, ConnackPacket) {

    info!("Connect the publisher");

    let server_addr = "localhost:1883".to_string();
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

    (stream, conn, connack)
}

pub fn publish_2( mut stream : &mut TcpStream, /*, conn : &mut ConnectPacket,*/ topic: &str, brightness: u8, state: &str ) {

    let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![(TopicFilter::new(topic).unwrap(), QualityOfService::Level0)];

    info!("Applying channel filters {:?} ...", channel_filters);
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let channels: Vec<TopicName> = vec![TopicName::new(topic).unwrap()];

    //let message = r#"{"brightness":100,"linkquality":189,"state":"ON"}"#.to_string();
    let message = format!("{{\"brightness\":{},\"state\":\"{}\"}}",  brightness, state);

    println!("Message : {}", &message);

    for chan in &channels {
        // let publish_packet = PublishPacket::new(chan.clone(), QoSWithPacketIdentifier::Level0, message.clone());
        let publish_packet = PublishPacketRef::new(chan, QoSWithPacketIdentifier::Level0, message.as_bytes());
        let mut buf = Vec::new();
        publish_packet.encode(&mut buf).unwrap();
        stream.write_all(&buf[..]).unwrap();

        // let (mut new_stream, _s, _a) = connect_publisher();
        // stream = &mut new_stream;
    }

}

pub fn _old_publish() {
    // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let matches = App::new("sub-client")
        .author("Y. T. Chung <zonyitoo@gmail.com>")
        .arg(
            Arg::with_name("SERVER")
                .short("S")
                .long("server")
                .takes_value(true)
                .required(true)
                .help("MQTT server address (host:port)"),
        )
        .arg(
            Arg::with_name("SUBSCRIBE")
                .short("s")
                .long("subscribe")
                .takes_value(true)
                .multiple(true)
                .required(true)
                .help("Channel filter to subscribe"),
        )
        .arg(
            Arg::with_name("USER_NAME")
                .short("u")
                .long("username")
                .takes_value(true)
                .help("Login user name"),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .short("p")
                .long("password")
                .takes_value(true)
                .help("Password"),
        )
        .arg(
            Arg::with_name("CLIENT_ID")
                .short("i")
                .long("client-identifier")
                .takes_value(true)
                .help("Client identifier"),
        )
        .get_matches();

    let server_addr = matches.value_of("SERVER").unwrap();
    let client_id = matches
        .value_of("CLIENT_ID")
        .map(|x| x.to_owned())
        .unwrap_or_else(generate_client_id);
    let channel_filters: Vec<(TopicFilter, QualityOfService)> = matches
        .values_of("SUBSCRIBE")
        .unwrap()
        .map(|c| (TopicFilter::new(c.to_string()).unwrap(), QualityOfService::Level0))
        .collect();

    let keep_alive = 10;

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

    info!("Applying channel filters {:?} ...", channel_filters);
    let sub = SubscribePacket::new(10, channel_filters);
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let channels: Vec<TopicName> = matches
        .values_of("SUBSCRIBE")
        .unwrap()
        .map(|c| TopicName::new(c.to_string()).unwrap())
        .collect();

    let user_name = matches.value_of("USER_NAME").unwrap_or("<anonym>");

    let mut cloned_stream = stream.try_clone().unwrap();
    /*
    thread::spawn(move || {
        loop {
            let packet = match VariablePacket::decode(&mut cloned_stream) {
                Ok(pk) => pk,
                Err(err) => {
                    error!("Error in receiving packet {:?}", err);
                    continue;
                }
            };
            trace!("PACKET {:?}", packet);

            match packet {
                VariablePacket::PingreqPacket(..) => {
                    let pingresp = PingrespPacket::new();
                    info!("Sending Ping response {:?}", pingresp);
                    pingresp.encode(&mut cloned_stream).unwrap();
                }
                VariablePacket::DisconnectPacket(..) => {
                    break;
                }
                _ => {
                    // Ignore other packets in pub client
                }
            }
        }
    });
    */


    // let stdin = io::stdin();
    //loop {
        // print!("{}: ", user_name);
        // io::stdout().flush().unwrap();
        //
        // let mut line = String::new();
        // stdin.read_line(&mut line).unwrap();
        //
        // if line.trim_end() == "" {
        //     continue;
        // }
        //
        // let message = format!("{}: {}", user_name, line.trim_end());

        let message = r#"{"brightness":10,"linkquality":189,"state":"OFF"}"#.to_string();

        for chan in &channels {
            // let publish_packet = PublishPacket::new(chan.clone(), QoSWithPacketIdentifier::Level0, message.clone());
            let publish_packet = PublishPacketRef::new(chan, QoSWithPacketIdentifier::Level0, message.as_bytes());
            let mut buf = Vec::new();
            publish_packet.encode(&mut buf).unwrap();
            stream.write_all(&buf[..]).unwrap();
        }
    //}
}
