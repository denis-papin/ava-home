// #![feature(proc_macro_hygiene, decl_macro)]
mod publish;
mod kitchen_lamp;
mod kitchen_inter_dim;
mod hall_inter_switch;
mod hall_lamp;
mod inside_temp_sensor;
mod kitchen_loop;
mod too_hot_loop;
mod stream;

extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate uuid;

use std::cell::{RefCell};

use std::{env};

use std::io::Write;
use std::net::TcpStream;

use std::str;
use std::sync::Arc;





use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};


use crate::hall_inter_switch::{InterSwitch};
use crate::hall_lamp::HallLampDevice;
use crate::inside_temp_sensor::InsideTempSensorDevice;

use crate::kitchen_inter_dim::{InterDim, KitchenInterDimDevice};
use crate::kitchen_lamp::{DeviceMessage, KitchenLampDevice, LampColor, LampRGB};
use crate::kitchen_loop::KitchenLoop;
use crate::too_hot_loop::TooHotLoop;
use crate::publish::{connect_publisher, publish};
use crate::stream::{ping_broker, wait_subpack};

fn generate_client_id() -> String {
    format!("/MQTT/rust/{}", Uuid::new_v4())
}

#[derive(Debug, Clone)]
pub struct Params {
    pub server_addr : String,
    pub client_id : String,
    pub channel_filters: Vec<(TopicFilter, QualityOfService)>,
    pub keep_alive :  u16,
}

fn device_to_listen() -> Vec<Box<dyn DynDevice>> {
    vec![Box::new(KitchenInterDimDevice::new()),
         Box::new(KitchenLampDevice::new()),
         Box::new(HallLampDevice::new()),
         Box::new(InsideTempSensorDevice::new()),
         // Box::new(HallInterSwitchDevice::new()),
    ]
}

fn parse_params() -> Params {
    let client_id = generate_client_id();

    let mut channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![];
    for dev in device_to_listen() {
        let topic = dev.get_topic();
        channel_filters.push((TopicFilter::new(topic).unwrap(), QualityOfService::Level0));
    }

    // let channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![
    //     (TopicFilter::new(format!("zigbee2mqtt/{KITCHEN_INTER_DIM}")).unwrap(), QualityOfService::Level0),
    //     (TopicFilter::new(format!("zigbee2mqtt/{}", KitchenLampDevice::get_name())).unwrap(), QualityOfService::Level0),
    //     (TopicFilter::new(format!("zigbee2mqtt/{HALL_INTER_SWITCH}")).unwrap(), QualityOfService::Level0),
    //     (TopicFilter::new(format!("zigbee2mqtt/{HALL_LAMP}")).unwrap(), QualityOfService::Level0),
    //     (TopicFilter::new(format!("zigbee2mqtt/{INSIDE_TEMP_SENSOR}")).unwrap(), QualityOfService::Level0),
    // ];

    Params {
        server_addr : "raspberrypi:1883".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
    }
}

pub (crate) trait DynDevice {
    fn get_topic(&self) -> String;
    fn is_init(&self) -> bool;
    fn init(&mut self, topic : &str, msg : &str, arc_locks: Arc<RefCell<Locks>>);
    fn execute(&self, topic : &str, msg : &str, pub_stream: &mut TcpStream, arc_locks: Arc<RefCell<Locks>>);

    /// Send the message on the right end point (/get) to trigger the device properties on the bus
    fn trigger_info(&self, pub_stream: &mut TcpStream);

    ///
    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage>;
    fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool,bool);
    fn forward_messages(&self,  pub_stream: &mut TcpStream, locks : &mut Locks, object_message : Box<dyn DeviceMessage>);
}

#[derive(Debug, Clone)]
pub (crate) struct DeviceLock<T> {
    count_locks : u32,
    last_object_message : T,
}

impl <T> DeviceLock<T> {
    pub (crate) fn inc(&mut self) {
        self.count_locks += 1;
    }
    pub (crate) fn dec(&mut self) {
        self.count_locks -= 1;
    }
    pub (crate) fn replace(&mut self, o : T) {
        self.last_object_message = o;
    }
}

#[derive(Debug, Clone)]
struct Locks {
    pub kitchen_inter_dim_lock : DeviceLock<InterDim>,
    pub kitchen_lamp_lock : DeviceLock<LampRGB>,
    pub hall_inter_switch : DeviceLock<InterSwitch>,
    pub hall_lamp_lock : DeviceLock<LampRGB>,
}

pub (crate) trait DynLoop {
    fn get_devices(&self) -> Vec<Box<dyn DynDevice>>;
}

pub (crate) fn find_loops(_topic : &str) -> Vec<Box<dyn DynLoop>> {
    vec![Box::new(KitchenLoop{})/*, Box::new(TooHotLoop{})*/]
}

// Devices we want to init before main processing
pub (crate) fn init_loop() -> Vec<Box<dyn DynDevice>> {
    vec![Box::new(KitchenLampDevice::new()),
         Box::new(HallLampDevice::new())]
}

///
///  Process incoming messages for initialization of devices
///
fn process_initialization_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream) -> Result<Locks, String> {

    let locks = Locks {
        kitchen_inter_dim_lock: DeviceLock {
            count_locks: 0,
            last_object_message: InterDim {
                brightness: 0,
                state: "".to_string()
            }
        },
        kitchen_lamp_lock: DeviceLock {
            count_locks: 0,
            last_object_message: LampRGB {
                color: LampColor {
                    hue: None,
                    saturation: None,
                    x: 0.0,
                    y: 0.0
                },
                brightness: 0,
                state: "".to_string()
            }
        },
        hall_inter_switch: DeviceLock {
            count_locks: 0,
            last_object_message: InterSwitch {
                state: "".to_string()
            }
        },
        hall_lamp_lock: DeviceLock {
            count_locks: 0,
            last_object_message: LampRGB {
                color: LampColor {
                    hue: None,
                    saturation: None,
                    x: 0.0,
                    y: 0.0
                },
                brightness: 0,
                state: "".to_string()
            }
        }
    };

    let arc_locks = Arc::new(RefCell::new(locks));
    let mut devices = init_loop();

    for dev in &devices {
        dev.trigger_info(&mut pub_stream);
    }

    loop {
        let mut end_loop = true;
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

                for dev in &mut devices {
                    dev.init(publ.topic_name(), msg, arc_locks.clone());
                }

                for dev in &devices {
                    info!("Devices before check ----------");

                    if ! dev.is_init() {
                        end_loop = false;
                    }
                }

            }
            _ => {}
        }

        if end_loop {
            break;
        }

    } // end while

    let borr = arc_locks.as_ref().borrow();
    let locks = borr.to_owned();

    Ok(locks)
}

fn process_incomming_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream, locks : Locks)  {

    let arc_locks = Arc::new(RefCell::new(locks));

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

                // TODO find the loop for the given topic
                let loops = find_loops(&publ.topic_name());

                for lp in loops {
                    let devices = lp.get_devices();
                    for dev in devices {
                        dev.execute(&publ.topic_name(), &msg, &mut pub_stream, arc_locks.clone());
                        // info!(">>>>>>>>>>> rc_locks after XXX {:?}", arc_locks.as_ref());
                    }
                }
            }
            _ => {}
        }
    }

}


fn main() {

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

    info!("Init devices");
    match process_initialization_message(&mut stream, &mut pub_stream) {
        Ok(locks) => {
            info!("Process incoming messages");
            let _ = process_incomming_message(&mut stream, &mut pub_stream, locks);
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}
