use std::any::Any;
use std::cell::{RefCell};
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::Arc;
use serde_derive::*;
use crate::{DeviceLock, DynDevice, HallLampDevice, InterSwitch, KitchenInterDimDevice, KitchenSwitchDevice, Locks, publish};
use crate::messages::{DeviceMessage, InterDim, LampRGB};


pub (crate) const KITCHEN_LAMP: &str = "kitchen_lamp";

#[derive(Debug)]
pub (crate) struct KitchenLampDevice {
    pub lock : Arc<RefCell<DeviceLock<String>>>,
    pub setup : bool,
}

impl KitchenLampDevice {
    pub(crate) fn new() -> Self {
        info!("🌟🌟🌟🌟🌟 NEW KitchenLampDevice");
        let dl = DeviceLock::new( String::new());
        Self {
            lock : Arc::new(RefCell::new( dl )),
            setup: false,
        }
    }

    pub fn get_name() -> &'static str {
        KITCHEN_LAMP
    }
}


impl DynDevice for KitchenLampDevice {

    fn get_lock(&self) -> Arc<RefCell<DeviceLock<String>>> {
        self.lock.clone()
    }

    fn setup(&mut self, setup: bool) {
        self.setup = setup;
    }

    fn get_topic(&self) -> String {
        format!("zigbee2mqtt/{}", Self::get_name())
    }

    fn is_init(&self) -> bool {
        self.setup
    }

    fn from_json_to_local(&self, msg: &str) -> Box<dyn DeviceMessage> {
        LampRGB::from_json(msg)
    }

    // TODO Generalize it
    fn init(&mut self, topic : &str, msg : &str, arc_locks: Arc<RefCell<Locks>>) {
        let locks = {
            let borr = arc_locks.as_ref().borrow();
            let mut locks = borr.deref().clone();

            if topic == &self.get_topic() {
                info!("✨ Init device {} :",  &self.get_topic().to_uppercase());
                let r_info: Result<LampRGB, _> = serde_json::from_str(msg);
                match r_info {
                    Ok(lamp) => {
                        self.setup = true;
                        locks.kitchen_lamp_lock.replace(lamp);
                    }
                    Err(e) => {
                        panic!("💀 Cannot parse the message for the device {}, {} :",  &self.get_topic().to_uppercase(), e);
                    }
                }
            }
            locks
        };
        arc_locks.replace(locks.clone());
    }

    fn read_object_message(&self, msg: &str) -> Box<dyn DeviceMessage> {
        let r_info: Result<LampRGB, _> = serde_json::from_str(msg);

        match r_info {
            Ok(lamp) => { Box::new(lamp) }
            Err(e) => {
                error!("💀 Cannot parse the message for device {}, e={}", &self.get_topic().to_uppercase(),  e);
                Box::new(LampRGB::new())
            }
        }
    }
    
    
    //
    fn allowed_to_process(&self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage>) -> (bool, bool) {
        let lamp_rgb = object_message.as_lamp_rgb();
        let is_locked = locks.kitchen_lamp_lock.count_locks > 0;
        let is_same = *lamp_rgb == locks.kitchen_lamp_lock.last_object_message;
        (is_locked, is_same)
    }

    fn replace( &self, locks : &mut Locks, object_message : &Box<dyn DeviceMessage> ) {
        let rgb = object_message.as_lamp_rgb().clone();
        locks.kitchen_lamp_lock.replace(rgb );
    }

    fn get_last_object_message_as_string(&self, locks : &mut Locks) -> String {
        format!( "{:?}", locks.kitchen_lamp_lock.last_object_message )
    }

    fn get_last_object_message(&self, locks : &mut Locks) -> Box<dyn DeviceMessage> {
        Box::new (locks.kitchen_lamp_lock.last_object_message.clone() )
    }

    fn lock(&self, locks : &mut Locks) {
        locks.kitchen_lamp_lock.inc();;
    }

    fn unlock(&self, locks : &mut Locks) {
        locks.kitchen_lamp_lock.dec();;
    }

    fn trigger_info(&self, mut pub_stream: &mut TcpStream) {
        publish(&mut pub_stream, &format!("{}/get", &self.get_topic()), r#"{"color":{"x":"","y":""}}"#);
    }

    fn to_local(&self, origin_message : &Box<dyn DeviceMessage>, last_message: &Box<dyn DeviceMessage>) -> Box<dyn DeviceMessage> {
        info!("KitchenLamp tries to build its LambRGB message");
        origin_message.to_lamp_rgb(&last_message)
    }
}

