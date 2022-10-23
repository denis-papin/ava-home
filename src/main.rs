// #![feature(proc_macro_hygiene, decl_macro)]
mod publish;
mod kitchen_lamp;
mod kitchen_inter_dim;
mod hall_inter_switch;
mod hall_lamp;
mod inside_temp_sensor;

extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate uuid;

use std::cell::{Cell, RefCell};
use serde_derive::*;
use std::{env, time};
use std::borrow::Borrow;
use std::io::Write;
use std::net::TcpStream;
use std::ops::Deref;
use std::str;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use clap::{App, Arg};

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};
use serde_json::Error;
use crate::DeviceType::{HallInterSwitch, HallLamp, InsideTempSensor, KitchenInterDim, KitchenLamp};
use crate::hall_inter_switch::{HallInterSwitchDevice, InterSwitch};
use crate::hall_lamp::HallLampDevice;
use crate::inside_temp_sensor::InsideTempSensorDevice;

use crate::kitchen_inter_dim::{InterDim, KitchenInterDimDevice};
use crate::kitchen_lamp::{KitchenLampDevice, LampColor, LampRGB};
use crate::LoopType::{KitchenLoop, TooHotLoop};
use crate::publish::{connect_publisher, publish};

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

const KITCHEN_LAMP: &str = "kitchen_lamp";
const HALL_LAMP : &str = "hall_lamp";
const KITCHEN_INTER_DIM : &str = "kitchen_inter_dim";
const HALL_INTER_SWITCH : &str = "hall_inter_switch";
const INSIDE_TEMP_SENSOR : &str = "inside_temp_sensor";


#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(TopicFilter, QualityOfService)>,
    pub keep_alive :  u16,
}

// TODO unify with connect_publisher()
fn parse_params() -> Params {
    let client_id = generate_client_id();

    let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![
        (TopicFilter::new("zigbee2mqtt/kitchen_inter_dim".to_string()).unwrap(), QualityOfService::Level0),
        (TopicFilter::new(format!("zigbee2mqtt/{KITCHEN_LAMP}")).unwrap(), QualityOfService::Level0),
        (TopicFilter::new("zigbee2mqtt/hall_inter_switch".to_string()).unwrap(), QualityOfService::Level0),
        (TopicFilter::new("zigbee2mqtt/hall_lamp".to_string()).unwrap(), QualityOfService::Level0),
        (TopicFilter::new(format!("zigbee2mqtt/{INSIDE_TEMP_SENSOR}")).unwrap(), QualityOfService::Level0),
    ];

    Params {
        server_addr : "raspberrypi:1883".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
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

#[derive(Debug)]
pub (crate) enum DeviceType {
    HallInterSwitch(HallInterSwitchDevice),
    KitchenInterDim(KitchenInterDimDevice),
    KitchenLamp(KitchenLampDevice),
    HallLamp(HallLampDevice),
    InsideTempSensor(InsideTempSensorDevice),
}



#[derive(Debug, Clone)]
struct Locks {
    pub dim_locks : u32,
    pub last_inter_dim : InterDim,

    pub lamp_locks : u32,
    pub last_kitchen_lamp : LampRGB,

    pub switch_locks: u32,
    pub last_inter_switch : InterSwitch,

    pub hall_lamp_locks : u32,
    pub last_hall_lamp : LampRGB,
}


// #[derive(Debug)]
enum LoopType {
    KitchenLoop(Vec<DeviceType>),
    TooHotLoop(Vec<DeviceType>),
}

struct Loop {
    // pub KITCHEN_LAMP : DeviceType,
    // pub KITCHEN_INTER_DIM : DeviceType,
    // pub HALL_INTER_SWITCH: DeviceType,
    pub kitchen_loop: LoopType,
    pub too_hot_loop: LoopType,
}

impl Loop {

    pub fn new() -> Self {
        let kitchen_lamp: DeviceType = KitchenLamp(KitchenLampDevice::new());
        let hall_lamp: DeviceType = HallLamp(HallLampDevice::new());
        let kitchen_inter_dim: DeviceType = KitchenInterDim(KitchenInterDimDevice::new());
        let hall_inter_switch: DeviceType = HallInterSwitch(HallInterSwitchDevice::new());


        let kitchen_loop: LoopType = KitchenLoop(vec![kitchen_inter_dim, kitchen_lamp, hall_lamp]);

        let inside_temp_sensor: DeviceType = InsideTempSensor(InsideTempSensorDevice::new());
        // let warning_lamp = WarningLamp(WarningLampDevice::new());
        let too_hot_loop: LoopType = TooHotLoop(vec![inside_temp_sensor]);

        Self {
            // KITCHEN_LAMP,
            // KITCHEN_INTER_DIM ,
            // HALL_INTER_SWITCH ,
            kitchen_loop,
            too_hot_loop,
        }
    }

    pub fn find_loops(&'_ self, topic : &str) -> Vec<&'_ LoopType> {
        vec![&self.kitchen_loop, &self.too_hot_loop]
    }

    // Devices we want to init before main processing
    pub fn init_loop() -> Vec<DeviceType> {
        let kitchen_lamp : DeviceType = KitchenLamp(KitchenLampDevice::new());
        let hall_lamp : DeviceType = HallLamp(HallLampDevice::new());
        vec![kitchen_lamp, hall_lamp]
    }

    pub fn execute(lt : &LoopType, topic : &str, msg : &str, mut pub_stream: &mut TcpStream,  arc_locks : Arc<RefCell<Locks>>) {

        match lt {
            KitchenLoop(deviceTypes) => {
                dbg!(&deviceTypes);
                for dt in deviceTypes {
                    match dt  {
                        HallInterSwitch(device) => {
                            device.execute(arc_locks.clone());
                        }
                        KitchenInterDim(device) => {
                            device.execute(topic, msg, &mut pub_stream, arc_locks.clone());
                            info!(">>>>>>>>>>> rc_locks after DIMMER {:?}", arc_locks.as_ref());

                        }
                        KitchenLamp(device) => {
                            device.execute(topic, msg, &mut pub_stream, arc_locks.clone());
                            info!(">>>>>>>>>>> rc_locks after LAMP {:?}", arc_locks.as_ref());
                        }
                        HallLamp(device) => {
                            device.execute(topic, msg, &mut pub_stream, arc_locks.clone());
                            info!(">>>>>>>>>>> rc_locks after HALL LAMP {:?}", arc_locks.as_ref());
                        }
                        InsideTempSensor(_) => {}
                    }

                }
            },
            TooHotLoop(deviceTypes) => {
                dbg!(&deviceTypes);
                for dt in deviceTypes {
                    match dt {
                        InsideTempSensor(device) => {
                            device.execute(topic, msg, &mut pub_stream, arc_locks.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

}

///
///  Process incoming messages for initialization of devices
///
fn process_initialization_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream) -> Result<Locks, String> {

    let mut locks = Locks {
        dim_locks: 0,
        last_inter_dim: InterDim { brightness: 0, state: "".to_string() },
        lamp_locks: 0,
        last_kitchen_lamp: LampRGB {
            color: LampColor {
                hue: None,
                saturation: None,
                x: 0.0,
                y: 0.0
            },
            brightness: 0,
            state: "".to_string()
        },
        switch_locks: 0,
        last_inter_switch: InterSwitch { state: "".to_string() },
        hall_lamp_locks: 0,
        last_hall_lamp: LampRGB {
            color: LampColor {
                hue: None,
                saturation: None,
                x: 0.0,
                y: 0.0
            },
            brightness: 0,
            state: "".to_string()
        }
    };

    publish(&mut pub_stream, &format!("zigbee2mqtt/{KITCHEN_LAMP}/get"), r#"{"color":{"x":"","y":""}}"#);
    publish(&mut pub_stream, "zigbee2mqtt/hall_lamp/get", r#"{"color":{"x":"","y":""}}"#);

    let mut end_loop = ( false, false);
    while end_loop != (true, true) {
        let packet = match VariablePacket::decode(&mut stream) {
            Ok(pk) => pk,
            Err(err) => {
                error!("Error in receiving packet {:?}", err);
                continue;
            }
        };

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

                let deviceTypes = Loop::init_loop();

                for dt in deviceTypes {
                    match dt  {
                        KitchenLamp(device) => {
                            match device.init(publ.topic_name(), msg) {
                                None => {}
                                Some(lamp_rgb) => {
                                    locks.last_kitchen_lamp = lamp_rgb;
                                    end_loop.0 = true;
                                }
                            }
                        }
                        HallLamp(device) => {
                            match device.init(publ.topic_name(), msg) {
                                None => {}
                                Some(lamp_rgb) => {
                                    locks.last_hall_lamp = lamp_rgb;
                                    end_loop.1 = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    } // end while

    Ok(locks)
}

fn process_incomming_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream, locks : Locks) -> Result<(), String> {

    let arc_locks = Arc::new(RefCell::new(locks));
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

                let loopTypes = dispatch_loop.find_loops(&publ.topic_name());

                for lt in loopTypes {
                    Loop::execute(lt, &publ.topic_name(), &msg, &mut pub_stream, arc_locks.clone());
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
    let mut pub_stream= connect_publisher(&params.server_addr);


    match process_initialization_message(&mut stream, &mut pub_stream) {
        Ok(locks) => {
            // Process incomming message from the dimmer
            let _ = process_incomming_message(&mut stream, &mut pub_stream, locks);
        }
        Err(e) => {
            panic!("{}", e);
        }
    }




}
