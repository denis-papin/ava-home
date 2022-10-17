// #![feature(proc_macro_hygiene, decl_macro)]
mod publish;

extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate uuid;

use serde_derive::*;
use std::env;
use std::io::Write;
use std::net::TcpStream;
use std::str;
use std::thread;
use std::time::{Duration, Instant};

use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};
use crate::DeviceType::{HallInterSwitch, KitchenInterDim, KitchenLamp};
use crate::LoopType::KitchenLoop;
use crate::publish::{connect_publisher, publish};

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

// "color":{"hue":4,
//          "saturation":97,
//          "x":0.640625,
//          "y":0.328125},
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct LampColor {
    hue: u32,
    saturation: u32,
    x:f32,
    y:f32,
}


// {"brightness":20,
// "color":{"hue":4,
//          "saturation":97,
//          "x":0.640625,
//          "y":0.328125},
// "color_mode":"xy",
// "color_temp":321,
// "color_temp_startup":300,
// "state":"OFF",
// "update":{"state":"idle"}}'
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct LampRGB {
    color : LampColor,
    brightness:u8,
    state: String,
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
struct LampBasic {
    //color : LampColor,
    brightness:u8,
    state: String,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct InterDim {
    brightness:u8,
    // linkquality:u8,
    state: String,
}

// {"consumption":0,
// "current":0,
// "device_temperature":32,
// "energy":0,
// "illuminance":0,
// "illuminance_lux":0,
// "linkquality":162,
// "power":0,
// "power_outage_count":1,
// "state":"ON",
// "voltage":240.2}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct InterSwitch {
    state: String,
}


#[derive(Debug, Clone)]
struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(TopicFilter, QualityOfService)>,
    pub keep_alive :  u16,
}


fn parse_params() -> Params {
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

    // let channel_filters: Vec<(TopicFilter, QualityOfService)> = matches
    //     .values_of("SUBSCRIBE")
    //     .unwrap()
    //     .map(|c| (TopicFilter::new(c.to_string()).unwrap(), QualityOfService::Level0))
    //     .collect();


    let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![
        (TopicFilter::new("zigbee2mqtt/kitchen_inter_dim".to_string()).unwrap(), QualityOfService::Level0),
        (TopicFilter::new("zigbee2mqtt/kitchen_lamp".to_string()).unwrap(), QualityOfService::Level0),
        (TopicFilter::new("zigbee2mqtt/hall_inter_switch".to_string()).unwrap(), QualityOfService::Level0),
    ];

    Params {
        server_addr : server_addr.to_string(),
        client_id,
        channel_filters,
        keep_alive : 10,
    }
}


fn wait_subpack(mut stream : &mut TcpStream) ->Result<(), String>   {
    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        if let VariablePacket::SubackPacket(ref ack) = packet {
            if ack.packet_identifier() != 10 {
                panic!("SUBACK packet identifier not match");
            }

            info!("Subscribed!");
            break;
        }
    }
    Ok(())
}

fn ping_broker(stream : &mut TcpStream, params : &Params) -> Result<(), String> {
    let params_clone = (*params).clone();
    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        let mut last_ping_time = Instant::now();
        let mut next_ping_time = last_ping_time + Duration::from_secs((params_clone.keep_alive as f32 * 0.9) as u64);
        loop {
            let current_timestamp = Instant::now();
            if params_clone.keep_alive > 0 && current_timestamp >= next_ping_time {
                info!("Sending PINGREQ to broker");

                let pingreq_packet = PingreqPacket::new();

                let mut buf = Vec::new();
                pingreq_packet.encode(&mut buf).unwrap();
                stream_clone.write_all(&buf[..]).unwrap();

                last_ping_time = current_timestamp;
                next_ping_time = last_ping_time + Duration::from_secs((params_clone.keep_alive as f32 * 0.9) as u64);
                thread::sleep(Duration::new((params_clone.keep_alive / 2) as u64, 0));
            }
        }
    });
    Ok(())
}


enum DeviceType {
    HallInterSwitch(HallInterSwitchDevice),
    KitchenInterDim(KitchenInterDimDevice),
    KitchenLamp(KitchenLampDevice),
}

struct HallInterSwitchDevice {

}

impl HallInterSwitchDevice {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn execute(&self) {
        info!(">>>>>>>>>> execute device SWITCH");
    }
}

struct KitchenInterDimDevice {

}

impl KitchenInterDimDevice {
    pub fn new() -> Self {
        Self {

        }
    }


    // TODO handle the locks
    //      Create a generic BasicDevice to make this routine general
    //      Build closure to process the publish to other devices
    pub fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream) {
        info!(">>>>>>>>>> execute device DIMMER");

        let mut dim_locks : u32 = 0;

        let mut last_inter_dim = InterDim {
            brightness: 0,
            // linkquality: 0,
            state: "".to_string()
        };

        if topic == "zigbee2mqtt/kitchen_inter_dim" {
            let r_info: Result<InterDim, _> = serde_json::from_str(msg);
            let inter_dim = r_info.unwrap();

            if dim_locks > 0 {
                info!("⛔ DIMMER MESSAGE Here we are, {:?} ", &inter_dim);
                info!("DIMMER IS LOCKED BY THE DIMMER ({}): {}", topic, msg);
                dim_locks -= 1;
            } else {

                if inter_dim == last_inter_dim {
                    info!("⛔ DIMMER [same message], {:?} ", &inter_dim);
                } else {
                    info!("🍺 DIMMER MESSAGE Here we are, {:?} ", &inter_dim);

                    // TODO lamp_locks += 1;
                    let lamp_rgb = LampBasic {
                        brightness: inter_dim.brightness,
                        state: inter_dim.state.clone(),
                    };

                    let message = serde_json::to_string(&lamp_rgb).unwrap();
                    publish(&mut pub_stream, "zigbee2mqtt/kitchen_lamp/set", &message);

                    // TODO switch_locks += 1;
                    let message = format!("{{\"state\":\"{}\"}}", &inter_dim.state);
                    publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                }

            }
            last_inter_dim = inter_dim;
        }


    }
}

struct KitchenLampDevice {
}

impl KitchenLampDevice {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn execute(&self, topic : &str, msg : &str, mut pub_stream: &mut TcpStream) {
        info!(">>>>>>>>>> execute device LAMP");
    }
}

enum LoopType {
    KitchenLoop(Vec<DeviceType>),
    HallLoop(Vec<DeviceType>),
}

struct Loop {
    // pub KITCHEN_LAMP : DeviceType,
    // pub KITCHEN_INTER_DIM : DeviceType,
    // pub HALL_INTER_SWITCH: DeviceType,
    pub KITCHEN_LOOP : LoopType,
}

// const KITCHEN_LAMP : DeviceType = KitchenLamp(KitchenLampDevice::new());
// const KITCHEN_INTER_DIM : DeviceType = KitchenInterDim(KitchenInterDimDevice::new());
// const HALL_INTER_SWITCH: DeviceType = HallInterSwitch(HallInterSwitchDevice::new());
// const KITCHEN_LOOP : LoopType = KitchenLoop(vec![KITCHEN_LAMP, KITCHEN_INTER_DIM, HALL_INTER_SWITCH]);

impl Loop {

    pub fn new() -> Self {
        let KITCHEN_LAMP : DeviceType = KitchenLamp(KitchenLampDevice::new());
        let KITCHEN_INTER_DIM : DeviceType = KitchenInterDim(KitchenInterDimDevice::new());
        let HALL_INTER_SWITCH: DeviceType = HallInterSwitch(HallInterSwitchDevice::new());
        let KITCHEN_LOOP : LoopType = KitchenLoop(vec![KITCHEN_LAMP, KITCHEN_INTER_DIM, HALL_INTER_SWITCH]);

        Self {
            // KITCHEN_LAMP,
            // KITCHEN_INTER_DIM ,
            // HALL_INTER_SWITCH ,
            KITCHEN_LOOP,
        }
    }

    pub fn find_loops(&'_ self, topic : &str) -> Vec<&'_ LoopType> {
        vec![&self.KITCHEN_LOOP]
    }

    pub fn execute(lt : &LoopType, topic : &str, msg : &str, mut pub_stream: &mut TcpStream) {
        info!(">>>>>>>>>> In the main loop");

        match lt {
            KitchenLoop(deviceTypes) => {
                for dt in deviceTypes {
                    match dt  {
                        HallInterSwitch(device) => {
                            device.execute();
                        }
                        KitchenInterDim(device) => {
                            device.execute(topic, msg, &mut pub_stream);
                        }
                        KitchenLamp(device) => {
                            device.execute(topic, msg, &mut pub_stream);
                        }
                    }
                }
            }
            LoopType::HallLoop(_) => {}
        }
    }

}


fn process_incomming_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream) -> Result<(), String> {

    let mut lamp_locks : u32 = 0;
    let mut dim_locks : u32 = 0;
    let mut switch_locks : u32 = 0;

    let mut last_inter_switch = InterSwitch {
        state: "".to_string(),
    };

    let mut last_inter_dim = InterDim {
        brightness: 0,
        // linkquality: 0,
        state: "".to_string()
    };

    let mut last_lamp_rgb = LampRGB {
        color: LampColor {
            hue: 0,
            saturation: 0,
            x: 0.0,
            y: 0.0
        },
        brightness: 0,
        state: "".to_string()
    };


    let dispatch_loop = Loop::new();

    loop {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {:?}", err);
                continue;
            }
        };
        trace!("PACKET {:?}", packet);

        match packet {
            VariablePacket::PingrespPacket(..) => {
                info!("Receiving PINGRESP from broker ..");
            }
            VariablePacket::PublishPacket(ref publ) => {
                let msg = match str::from_utf8(publ.payload()) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to decode publish message {:?}", err);
                        continue;
                    }
                };
                info!("PUBLISH ({}): {}", publ.topic_name(), msg);

                // ---

                    let loopTypes = dispatch_loop.find_loops(&publ.topic_name());

                    for lt in loopTypes {
                        Loop::execute(lt, &publ.topic_name(), &msg, &mut pub_stream);
                    }
                //

                // if (publ.topic_name()) == "zigbee2mqtt/kitchen_inter_dim" {
                //     let r_info: Result<InterDim, _> = serde_json::from_str(msg);
                //     let inter_dim = r_info.unwrap();
                //
                //     if dim_locks > 0 {
                //         info!("⛔ DIMMER MESSAGE Here we are, {:?} ", &inter_dim);
                //         info!("DIMMER IS LOCKED BY THE DIMMER ({}): {}", publ.topic_name(), msg);
                //         dim_locks -= 1;
                //     } else {
                //
                //         if inter_dim == last_inter_dim {
                //             info!("⛔ DIMMER [same message], {:?} ", &inter_dim);
                //         } else {
                //             info!("🍺 DIMMER MESSAGE Here we are, {:?} ", &inter_dim);
                //
                //             lamp_locks += 1;
                //             let lamp_rgb = LampBasic {
                //                 brightness: inter_dim.brightness,
                //                 state: inter_dim.state.clone(),
                //             };
                //
                //             let message = serde_json::to_string(&lamp_rgb).unwrap();
                //             publish(&mut pub_stream, "zigbee2mqtt/kitchen_lamp/set", &message);
                //
                //             switch_locks += 1;
                //             let message = format!("{{\"state\":\"{}\"}}", &inter_dim.state);
                //             publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                //         }
                //
                //     }
                //     last_inter_dim = inter_dim;
                // }


                if (publ.topic_name()) == "zigbee2mqtt/kitchen_lamp" {

                    let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
                    let lamp_rgb = r_info.unwrap();

                    if lamp_locks > 0 {
                        info!("⛔ LAMP Here we are, {:?} ", &lamp_rgb);
                        info!("LAMP IS LOCKED BY THE DIMMER ({}): {}", publ.topic_name(), msg);
                        lamp_locks -= 1;
                    } else {

                        if lamp_rgb == last_lamp_rgb {
                            info!("⛔ LAMP [same message], {:?} ", &lamp_rgb);
                        } else {
                            info!("🍺 LAMP Here we are, {:?} ", &lamp_rgb);
                            info!("PROCESS LAMP ({}): {}", publ.topic_name(), msg);
                            dim_locks += 1;
                            let message = format!("{{\"brightness\":{},\"state\":\"{}\"}}", lamp_rgb.brightness, &lamp_rgb.state);
                            publish(&mut pub_stream, "zigbee2mqtt/kitchen_inter_dim/set", &message);

                            switch_locks += 1;
                            let message = format!("{{\"state\":\"{}\"}}", &lamp_rgb.state);
                            publish(&mut pub_stream, "zigbee2mqtt/hall_inter_switch/set", &message);
                        }
                        last_lamp_rgb = lamp_rgb;
                    }
                }


                if (publ.topic_name()) == "zigbee2mqtt/hall_inter_switch" {
                    let r_info: Result<InterSwitch, _> = serde_json::from_str(msg);
                    let inter_switch = r_info.unwrap();

                    if switch_locks > 0 {
                        info!("⛔ SWITCH Here we are, {:?} ", &inter_switch);
                        info!("SWITCH IS LOCKED BY THE DIMMER ({}): {}", publ.topic_name(), msg);
                        switch_locks -= 1;
                    } else {

                        if inter_switch == last_inter_switch {
                            info!("⛔ SWITCH [same message], {:?} ", &inter_switch);
                        } else {
                            info!("🍺 SWITCH Here we are, {:?} ", &inter_switch);
                            info!("PROCESS SWITCH ({}): {}", publ.topic_name(), msg);
                            dim_locks += 1;
                            let message = format!("{{\"state\":\"{}\"}}", &inter_switch.state);
                            publish(&mut pub_stream, "zigbee2mqtt/kitchen_inter_dim/set", &message);

                            lamp_locks += 1;
                            let message = format!("{{\"state\":\"{}\"}}", &inter_switch.state);
                            publish(&mut pub_stream, "zigbee2mqtt/kitchen_lamp/set", &message);
                        }

                    }
                    last_inter_switch = inter_switch;
                }

            }
            _ => {}
        }
    }
    Ok(())
}


fn main() {
    // publish_2( "zigbee2mqtt/kitchen_lamp/set", 120, "ON" );
    // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let params = parse_params();

    info!("Connecting to mqtt at {:?} ... ", &params.server_addr);
    let mut stream = TcpStream::connect(&params.server_addr).unwrap();
    info!("Connected!");

    info!("Client identifier {:?}", &params.client_id);
    let mut conn = ConnectPacket::new(&params.client_id);
    conn.set_clean_session(true);
    conn.set_keep_alive(params.keep_alive);
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

    info!("Applying channel filters {:?} ...", &params.channel_filters);
    let sub = SubscribePacket::new(10, params.channel_filters.clone());
    let mut buf = Vec::new();
    sub.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let _ = wait_subpack(&mut stream);
    let _ = ping_broker(&mut stream, &params);

    // Open the stream to publish the response(s)
    let mut pub_stream= connect_publisher();

    // Process incomming message from the dimmer
    let _ = process_incomming_message(&mut stream, &mut pub_stream);
}
