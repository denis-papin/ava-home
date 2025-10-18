use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use log::{error, info};
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::QoS;
use serde::de::DeserializeOwned;
use crate::device_lock::DeviceLock;

pub const ZIGBEE_FAMILY : &str = "zigbee2mqtt";
pub const EXTERNAL_FAMILY: &str = "external";
pub const SYSTEM_FAMILY: &str = "regulator";
/// A Locality is a set of features
/// shared by a group of messages often called a MessageEnum

pub trait Locality : Clone + Debug  {
    // Self is MessageEnum indeed
    fn query_for_state(&self) -> String;
    fn find_set_topic(&self, topic: &str) -> String;
    fn raw_message(&self) -> String;
    fn to_local(&self, original_message: &Self, last_message: &Self) -> Self;
    fn to_local_with_data(&self, original_message: &Self, last_message: &Self, ext_data: Option<&HashMap<String, f64>>, topic: Option<&str>) -> Self;
    fn json_to_local(&self, json_msg: &str) -> Result<Self, String>;
    fn process(&self, topic: &str, args: &[String]) -> impl Future<Output = ()> + Send;
    fn compute(&self) -> impl Future<Output = Option<HashMap<String, f64>>> + Send;
}


#[derive(Debug)]
pub struct GenericDevice<T: Locality> {
    pub family: String, // "zigbee2mqtt", "regulator", "external", ...
    pub name: String,
    pub message_type: T,
    pub lock: Arc<RefCell<DeviceLock<T>>>,
    pub setup: bool,
    pub process_same_message: bool,
}

impl <T> GenericDevice<T>  where T : Locality + DeserializeOwned {

    pub fn new(family: &str, name : &str, msg: T, process_same_message: bool) -> Self {
        info!("🌟 New Generic Device, topic = [{}]", &name);
        let dl = DeviceLock::new(msg.clone());
        Self {
            family: family.to_string(),
            name: name.to_string(),
            message_type: msg,
            lock: Arc::new(RefCell::new(dl)),
            setup: false,
            process_same_message
        }
    }

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<T>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    pub fn make_topic(family: &str, device_name: &str) -> String {
        format!("{}/{}", family, device_name)
    }

    pub fn get_topic(&self) -> String {
        format!("{}/{}", self.family, self.name)
    }
    pub fn is_init(&self) -> bool {
        self.setup
    }

    pub fn init(&mut self, topic : &str, json_msg: &str) {
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();

            if topic == &self.get_topic() {
                match dev_lock.last_object_message.json_to_local(json_msg) {
                    Ok(msg) => {
                        info!("✨ Init device [{}], with message <{:?}>",  &self.get_topic().to_uppercase(), &msg);
                        //self.topic = topic.to_string();
                        self.setup(true);
                        dev_lock.replace(msg);
                        info!("Init done");
                    }
                    Err(e) => {
                        error!("✨ Fail to convert init message for device [{}], e=[{}]>",  &self.get_topic().to_uppercase(), e);
                    }
                }
            }
            dev_lock
        };
        self.get_lock().replace(new_lock.clone());
    }

    /// Send the message on the right end point (/get) to trigger the device properties on the bus
    pub fn trigger_info(&self) -> Vec<u8> {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();
        dev_lock.last_object_message.query_for_state().as_bytes().to_vec()
    }

    fn allowed_to_process(&self, object_message: &T) -> (bool, bool) {
        let lk = self.get_lock();
        let borr = lk.as_ref().borrow();
        let dev_lock = borr.deref().clone();

        let incoming_message = object_message.raw_message();
        let is_locked = dev_lock.count_locks > 0;
        let is_same = *incoming_message == dev_lock.last_object_message.raw_message();
        (is_locked, is_same)
    }

    ///
    /// Specific processing for the device that emits the message
    ///
    async fn process(&self,  original_message : &T, args: &[String]) {
        info!("Default empty process for device {}.", & self.get_topic());
        original_message.process(& self.get_topic(), &args).await;
    }

    ///
    /// Run the local specific processing if allowed.
    ///
    pub async fn process_and_continue(&self, original_message : &T, args: &[String]) -> bool {

        info!("process_and_continue");
        let (new_lock, allowed) = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();
            let allowed: bool;
            match self.allowed_to_process(&original_message) {
                (true, _) => {
                    info!("❌ Device {} is locked.", & self.get_topic().to_uppercase());
                    dev_lock.dec();
                    allowed = false;
                }
                (false, true) => {
                    if self.process_same_message {
                        info!("❌ Device {}, same message, process anyways.", & self.get_topic().to_uppercase());
                        self.process(&original_message, &args).await; // In this case, we process the message even if it's the same as before
                        allowed = true;    
                    } else {
                        info!("❌ Device {}, same message.", & self.get_topic().to_uppercase());
                        allowed = false;
                    }
                }
                (false, false) => {
                    info!("👍 Device {}, allowed to process the message.", & self.get_topic().to_uppercase());
                    self.process(&original_message, &args).await;
                    allowed = true;
                }
            }
            //let json_message= original_message.raw_message();
            dev_lock.replace(original_message.clone());
            (dev_lock, allowed)
        };
        self.get_lock().replace(new_lock);
        allowed
    }

    ///
    /// Make the device consume the current message
    ///
    pub async fn consume_message(&self, original_message : &T, o_ext_data: Option<&HashMap<String, f64>>, mut client: &mut AsyncClient) {
        info!("The device is consuming the message");
        let new_lock = {
            let lk = self.get_lock();
            let borr = lk.as_ref().borrow();
            let mut dev_lock = borr.deref().clone();

            info!("Execute device {}", & self.get_topic().to_uppercase());

            // Last message est du même format que le message du device.
            // Il permet de récupérer certaines informations.
            // Ex : Incoming inter dim message + last (LampRGB) ---> hall_lamp message (LampRGB)
            // In Generic Mode it's much simplier, we have the last message in the correct format.
            let last_message = &dev_lock.last_object_message;

            // TODO : make the get_topic an option here
            let topic = self.get_topic();
            let o_topic = match topic.as_str() {
                "" => None,
                t => Some(t),
            };
            
            let object_message = self.message_type.to_local_with_data(&original_message, &last_message, o_ext_data, o_topic );
            // let object_message = self.message_type.to_local(&original_message, &last_message);
            // let object_message = self.to_local(&original_message, &last_message);

            match self.allowed_to_process(&object_message) {
                (true, _) => {
                    info!("⛔ Device {} is locked.", & self.get_topic().to_uppercase());
                    info!("object message : {:?}", &object_message);
                    info!("Last message : {:?}", &dev_lock.last_object_message);
                    dev_lock.dec();
                    // self.unlock(&mut locks);
                }
                (false, true) => {
                    info!("⛔ Device {}, same message.", & self.get_topic().to_uppercase());
                    info!("object message : {:?}", &object_message);
                    info!("Last message : {:?}", &dev_lock.last_object_message);
                }
                (false, false) => {
                    info!("🍺 Device {}, process the message.", & self.get_topic().to_uppercase());
                    info!("object message : {:?}", &object_message);
                    info!("Last message : {:?}", &dev_lock.last_object_message);
                    dev_lock.inc();
                    self.publish_message(&mut client, &object_message).await;
                }
            }
            dev_lock.replace(object_message);

            let message_locked = &dev_lock.last_object_message;
            info!("Now last : {:?}", &message_locked);
            dev_lock
        };
        self.get_lock().replace(new_lock);
    }
    
    pub async fn publish_message(&self, client: &mut AsyncClient, object_message : &T) {
        let message = object_message.raw_message();
        let data = message.as_bytes().to_vec();
        let set_topic = self.message_type.find_set_topic(&self.get_topic());
        info!("Publishing the message to channel [{}]", & set_topic);
        client.publish(&set_topic, QoS::AtLeastOnce, false, data).await.unwrap(); // TODO
    }
}
