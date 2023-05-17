// #![feature(proc_macro_hygiene, decl_macro)]
mod publish;
mod kitchen_lamp;
mod kitchen_inter_dim;
mod devices {}
mod stream;
mod messages;
mod outdoor_temp_sensor;
mod kitchen_switch;
mod hall_lamp;
mod inside_temp_sensor;

extern crate mqtt;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate uuid;

use std::cell::{RefCell};

use std::{env, thread, time};
use std::collections::HashMap;
use std::fmt::Debug;

use std::io::Write;
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};

use std::str;
use std::sync::Arc;

use uuid::Uuid;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicFilter;
use mqtt::{Decodable, Encodable, QualityOfService};

use crate::hall_lamp::{HALL_LAMP, HallLampDevice};
use crate::inside_temp_sensor::{InsideTempSensorDevice, TEMP_BAIE_VITREE};


use crate::kitchen_inter_dim::{KITCHEN_INTER_DIM, KitchenInterDimDevice};
use crate::kitchen_lamp::{KITCHEN_LAMP, KitchenLampDevice};

use crate::messages::{DeviceMessage, InterSwitch};
use crate::outdoor_temp_sensor::{OutdoorTempSensorDevice, TEMP_MEUBLE_TV};

use crate::publish::{connect_publisher, publish};
use crate::stream::{ping_broker, wait_subpack};

pub (crate) const KITCHEN_LOOP : &str = "KITCHEN_LOOP";
pub (crate) const TOO_HOT_LOOP : &str = "TOO_HOT_LOOP";
pub (crate) const SENSOR_LOOP : &str = "SENSOR_LOOP";

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

fn device_to_listen(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<Arc<RefCell<dyn DynDevice>>> {
    vec![
        device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone(),
        device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
        device_repo.get(TEMP_MEUBLE_TV).unwrap().clone()
        // device_repo.get(KITCHEN_SWITCH).unwrap().clone(),
    ]
}


pub (crate) fn find_loops(topic: &str, all_loops: &mut Vec<HardLoop>) -> (Vec<HardLoop>, Option<Arc<RefCell<dyn DynDevice>>>)  {
    let mut eligible_loops : Vec<HardLoop> = vec![];
    let mut output_dev : Option<Arc<RefCell<dyn DynDevice>>> = None;

    for lp in all_loops {
        match lp.find_device_by_topic(topic) {
            None => {}
            Some(dev) => {
                info!("Found topic in [{}] loop, topic=[{}]", & lp.get_name(), topic);
                eligible_loops.push(lp.clone());
                output_dev = Some(dev.clone());
            }
        }

    }
    (eligible_loops, output_dev)
}

// TODO Don't listen to the devices here !
fn parse_params(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Params {
    let client_id = generate_client_id();

    let mut channel_filters: Vec<(TopicFilter, QualityOfService)> = vec![];
    for dev in device_to_listen(&device_repo) {
        let dd = dev.as_ref().borrow();
        let topic = dd.get_topic();
        channel_filters.push((TopicFilter::new(topic).unwrap(), QualityOfService::Level0));
    }

    Params {
        server_addr : "raspberrypi:1883".to_string(),
        client_id,
        channel_filters,
        keep_alive : 30_000,
    }
}

pub (crate) trait DynDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        todo!()
    }

    fn setup(&mut self, _setup: bool) {
        todo!()
    }

    fn get_topic(&self) -> String;
    fn is_init(&self) -> bool;

    fn init(&mut self, topic : &str, msg : &str) {
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            if topic == &self.get_topic() {
                info!("✨ Init device [{}], with message <{}>",  &self.get_topic().to_uppercase(), &msg);
                self.setup(true);
                dev_lock.replace(msg.to_string());
                info!("Init done");
            }
            dev_lock
        };
        self.get_lock().replace(new_lock.clone());
    }

    /// Send the message on the right end point (/get) to trigger the device properties on the bus
    fn trigger_info(&self, pub_stream: &mut TcpStream);

    fn from_json_to_local(&self, msg: &str) -> Result<Box<dyn DeviceMessage>, String>;

    // fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage>;
    // fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool,bool);

    fn allowed_to_process(&self, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();

        let incoming_message = object_message.to_json().unwrap();
        let is_locked = dev_lock.count_locks > 0;
        let is_same = *incoming_message == dev_lock.last_object_message;
        (is_locked, is_same)
    }

    ///
    /// Specific processing for the device that emits the message
    ///
    fn process(&self,  _original_message : &Box<dyn DeviceMessage>) {
        // Nothing by defaut
        info!("Default empty process for device {}.", & self.get_topic());
    }

    ///
    /// Run the local specific processing if allowed.
    ///
    fn process_and_continue(&self, original_message : &Box<dyn DeviceMessage>) -> bool {

        info!("process_and_continue");
        let (new_lock, allowed) = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            let allowed: bool;
            match self.allowed_to_process(&original_message) {
                (true, _) => {
                    info!("❌ Device {} is locked.", & self.get_topic().to_uppercase());
                    // self.unlock(&mut locks);
                    dev_lock.dec();
                    allowed = false;
                }
                (false, true) => {
                    info!("❌ Device {}, same message.", & self.get_topic().to_uppercase());
                    allowed = false;
                }
                (false, false) => {
                    info!("👍 Device {}, allowed to process the message.", & self.get_topic().to_uppercase());
                    self.process(&original_message);
                    allowed = true;
                }
            }
            let json_message = original_message.to_json().unwrap().clone();
            dev_lock.replace(json_message);
            (dev_lock, allowed)
        };
        self.get_lock().replace(new_lock);
        allowed
    }

    ///
    /// Make the device consume the current message
    ///
    fn consume_message(&self, original_message : &Box<dyn DeviceMessage>, mut pub_stream: &mut TcpStream) {
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();

            info!("Execute device {}", & self.get_topic().to_uppercase());

            // Convert the incoming message to the format the device needs. Last message est du même format que le message du device. Il permet de récupérer certaines informations.
            // Ex : Incoming inter dim message + last (LampRGB) ---> hall_lamp message (LampRGB)
            let last_message = match self.from_json_to_local(&dev_lock.last_object_message)  {
                Err(e) => {
                    error!("💀 Cannot parse the message for device {}, message=<{}>, \n e={}", &self.get_topic().to_uppercase(), &dev_lock.last_object_message, e);
                    return;
                }
                Ok(lm) => lm
            };
            let object_message = self.to_local(&original_message, &last_message);

            dbg!(&object_message.to_json());

            match self.allowed_to_process(&object_message) {
                (true, _) => {
                    info!("⛔ Device {} is locked.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                    dev_lock.dec();
                    // self.unlock(&mut locks);
                }
                (false, true) => {
                    info!("⛔ Device {}, same message.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                }
                (false, false) => {
                    info!("🍺 Device {}, process the message.", & self.get_topic().to_uppercase());
                    info!("Incoming message : {:?}, last message : {:?}", &object_message.to_json(), &dev_lock.last_object_message);
                    // self.lock(&mut locks);
                    dev_lock.inc();
                    self.publish_message(&mut pub_stream, &object_message);
                }
            }
            let json_message = object_message.to_json().unwrap().clone();
            dev_lock.replace(json_message);

            let message_locked = &dev_lock.last_object_message; // self.get_last_object_message_as_string(&mut locks);
            info!("Now last : {:?}", &message_locked);
            dev_lock
        };
        self.get_lock().replace(new_lock);
    }


    fn publish_message(&self, mut pub_stream: &mut TcpStream, object_message : &Box<dyn DeviceMessage>) {
        match object_message.to_json() {
            Ok(message) => {
                info!("➡ Prepare to be sent to the {}, {:?} ", &self.get_topic().to_uppercase(), &message);
                publish(&mut pub_stream, &format!("{}/set", &self.get_topic()), &message);
            }
            Err(e) => {
                error!("💣 Impossible to parse the message : e={:?}", e);
            }
        }
    }


    // Could be a method of a receiver trait
    fn receive(&self, mut pub_stream: &mut TcpStream, object_message : Box<dyn DeviceMessage>) {
        match object_message.to_json() {
            Ok(message) => {
                info!("➡ Prepare to be sent to the {}, {:?} ", &self.get_topic().to_uppercase(), &message);
                publish(&mut pub_stream, &format!("{}/set", &self.get_topic()), &message);
            }
            Err(e) => {
                error!("💣 Impossible to parse the message : e={:?}", e);
            }
        }
    }

    // Convert any message (origin_message) into a local message type needed by the device
    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage>;
}

#[derive(Debug, Clone)]
pub (crate) struct DeviceLock<T> {
    count_locks : u32,
    last_object_message : T,
}

impl <T> DeviceLock<T> {
    pub (crate) fn new(last_message: T) -> Self {
        Self {
            count_locks: 0,
            last_object_message: last_message,
        }
    }

    pub (crate) fn inc(&mut self) {
        self.count_locks += 1;
        info!("🔼 After up Locks:[{}]", self.count_locks);
    }
    pub (crate) fn dec(&mut self) {
        self.count_locks -= 1;
        info!("⏬After down Locks:[{}]", self.count_locks);
    }

    pub (crate) fn replace(&mut self, o : T) {
        self.last_object_message = o;
    }

}

#[derive(Clone)]
pub (crate) struct HardLoop {
    pub name : String,
    pub devices : Vec<Arc<RefCell<dyn DynDevice>>>,
}

impl HardLoop {
    fn new(name: String, devices : Vec<Arc<RefCell<dyn DynDevice>>>) -> Self {
        Self {
            name,
            devices,
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_devices(&self) -> Vec<Arc<RefCell<dyn DynDevice>>> {
        self.devices.clone()
    }

    fn find_device_by_topic(&self, topic: &str) -> Option<Arc<RefCell<dyn DynDevice>>> {
        for dev in self.get_devices() {
            let dd = dev.deref().borrow();
            if dd.get_topic() == topic {
                return Some(dev.clone());
            }
        }
        None
    }

    fn loop_devices(&self, topic: &str, original_message: &Box<dyn DeviceMessage>, mut pub_stream: &mut TcpStream) {
        for dev in self.get_devices() {
            let dd1 = dev.as_ref().borrow();
            let dd = dd1.deref();
            if &dd.get_topic() != topic {
                info!("🚀 Device Topic of the loop: [{:?}]", &dd.get_topic());
                dd.consume_message(&original_message, &mut pub_stream);
                info!("🚩 End Device Topic of the loop: [{:?}]", &dd.get_topic());
            }
        }
    }

}

pub (crate) trait DynLoop {
    fn get_name(&self) -> &'static str;
    fn get_devices(&self) -> Vec<Box<dyn DynDevice>>;
    fn has_topic(&self, topic: &str) -> bool {
        for dev in self.get_devices() {
            if dev.get_topic() == topic {
                return true;
            }
        }
        false
    }

    fn find_device_by_topic(&self, topic: &str) -> Option<Box<dyn DynDevice>> {
        for dev in self.get_devices() {
            if dev.get_topic() == topic {
                let _ = dev.deref();
                return Some(dev);
            }
        }
        None
    }

}

fn build_device_repo() -> HashMap<String, Arc<RefCell<dyn DynDevice>>> {
    info!("Inside the Repo Builder");
    let mut device_repo : HashMap<String, Arc<RefCell<dyn DynDevice>>> = HashMap::new();
    device_repo.insert(KITCHEN_INTER_DIM.to_owned(), Arc::new(RefCell::new(KitchenInterDimDevice::new())));
    device_repo.insert(KITCHEN_LAMP.to_owned(), Arc::new(RefCell::new(KitchenLampDevice::new())));
    device_repo.insert(HALL_LAMP.to_owned(), Arc::new(RefCell::new(HallLampDevice::new())));
    device_repo.insert(TEMP_BAIE_VITREE.to_owned(), Arc::new(RefCell::new(InsideTempSensorDevice::new())));
    device_repo.insert(TEMP_MEUBLE_TV.to_owned(), Arc::new(RefCell::new(OutdoorTempSensorDevice::new())));
    device_repo
}


fn build_loops(device_repo: &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<HardLoop> {

    let kitchen_loop = HardLoop::new( KITCHEN_LOOP.to_string(),
    vec![
        device_repo.get(KITCHEN_INTER_DIM).unwrap().clone(),
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone(),
    ]);

    let too_hot_loop = HardLoop::new( TOO_HOT_LOOP.to_string(),
    vec![
        device_repo.get(TEMP_BAIE_VITREE).unwrap().clone(),
    ]);

    let sensor_loop = HardLoop::new( SENSOR_LOOP.to_string(),
                                      vec![
                                          device_repo.get(TEMP_MEUBLE_TV).unwrap().clone(),
                                      ]);

    vec![kitchen_loop, too_hot_loop, sensor_loop]
}

/// Build the list of devices to be initialized
fn build_init_list(device_repo : &HashMap<String, Arc<RefCell<dyn DynDevice>>>) -> Vec<Arc<RefCell<dyn DynDevice>>> {
    vec![
        device_repo.get(KITCHEN_LAMP).unwrap().clone(),
        device_repo.get(HALL_LAMP).unwrap().clone()
    ]
}

///
/// Send an information message for all the device we want to init
/// Read the responses from mosquitto and run the init routine for the devices.
///
fn process_initialization_message(mut stream : &mut TcpStream, mut pub_stream: &mut TcpStream, device_to_init: &Vec<Arc<RefCell<dyn DynDevice>>>) -> Result<(), String> {

    info!("Init devices");

    if !device_to_init.is_empty() {
        for dev in device_to_init {
            let borr = dev.as_ref().borrow();
            let dd = borr.deref().clone();

            dbg!("IN DEV2", &dd.get_topic());
            dd.trigger_info(&mut pub_stream);
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

                    for dev in device_to_init {
                        let mut borr = dev.as_ref().borrow_mut();
                        let dd = borr.deref_mut();
                        dd.init(publ.topic_name(), msg);
                    }

                    for dev in device_to_init {
                        info!("Devices before check ----------");

                        let borr = dev.as_ref().borrow();
                        let dd = borr.deref().clone();

                        if !dd.is_init() {
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
    } // device is empty

    info!("Initialisation stage finished");

    Ok(())
}

///
///
///
fn process_incoming_message(mut stream: &mut TcpStream, mut pub_stream: &mut TcpStream, mut all_loops: &mut Vec<HardLoop>)  {
    let ten_millis = time::Duration::from_millis(10);
    loop {
        info!("** New Round **");
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

                let (loops, opt_device) = find_loops(&publ.topic_name(), &mut all_loops);

                match opt_device {
                    None => {
                        info!("No device to process the message");
                    }
                    Some(dev) => {
                        info!("Receiver device found !");
                        let dd1 = dev.as_ref().borrow();
                        let dd = dd1.deref();
                        for lp in loops {
                            info!("Before Looping");
                            // Change the msg into the DeviceMessage box of the ad hoc device (the original device)

                            let original_message = match dd.from_json_to_local(msg) {
                                Ok(om) => {om}
                                Err(e) => {
                                    error!("💀 Cannot parse the message locally for device {}, msg=<{}>, \n e={}", &dd.get_topic().to_uppercase(), msg, e);
                                    continue
                                }
                            };

                            if dd.process_and_continue(&original_message) {
                                lp.loop_devices(&publ.topic_name(), &original_message, &mut pub_stream);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        thread::sleep(ten_millis);
    }
}


fn main() {

    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    info!("Starting AVA");

    info!("Building the device repository");
    let device_repo = build_device_repo();
    let params = parse_params(&device_repo);

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


    let mut init_list = build_init_list(&device_repo);
    let mut all_loops = build_loops(&device_repo);

    match process_initialization_message(&mut stream, &mut pub_stream, &mut init_list) {
        Ok(_) => {
            info!("Process incoming messages");
            let _ = process_incoming_message(&mut stream, &mut pub_stream, &mut all_loops);
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}
